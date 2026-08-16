use crate::aws::AwsClient;
use crate::aws::aws_qualifier::AwsFunctionQualifier;
use crate::detect::{FunctionToDeploy, PluginFunctionToDeploy};
use crate::errors::DeployError;
use crate::kn::KnClient;
use crate::kn::kn_qualifier::KnFunctionQualifier;
use crate::repository::DeployerRepository;
use crate::repository::create_deployment_params::{
    AwsFunctionToCreate, CreateDeploymentParams, FunctionToCreate, KnFunctionToCreate,
    PluginFunctionToCreate,
};
use anyhow::{Context, Result};
use florca_core::deployment::{DeploymentEntity, DeploymentName};
use florca_core::function::{FunctionConfig, FunctionEntity};
use std::fs::File;
use std::{io::Write, path::Path, sync::Arc};
use tempfile::TempDir;
use tracing::info;
use zip::ZipArchive;

#[derive(Debug, Clone)]
pub struct Deployer {
    pub repository: Arc<dyn DeployerRepository>,
    pub aws_client: Arc<dyn AwsClient>,
    pub kn_client: Arc<dyn KnClient>,
}

impl Deployer {
    pub fn new(
        repository: Arc<dyn DeployerRepository>,
        aws_client: Arc<dyn AwsClient>,
        kn_client: Arc<dyn KnClient>,
    ) -> Self {
        Self {
            repository,
            aws_client,
            kn_client,
        }
    }

    pub async fn deploy(
        &self,
        bytes: &[u8],
        deployment_name: &DeploymentName,
        force: bool,
    ) -> Result<(), DeployError> {
        let mut zip_file = tempfile::tempfile()?;
        zip_file.write_all(bytes)?;
        let temp_deployment_dir = extract_zip(&zip_file)?;
        self.deploy_dir(temp_deployment_dir.path(), deployment_name, force)
            .await?;
        info!(
            deployment = deployment_name.to_string(),
            "Deployment successful"
        );
        Ok(())
    }

    async fn deploy_dir(
        &self,
        source_deployment_path: &Path,
        deployment_name: &DeploymentName,
        force: bool,
    ) -> Result<(), DeployError> {
        let functions_to_deploy: Vec<FunctionToDeploy> =
            crate::detect::detect_functions(source_deployment_path).await?;
        validate_kn_names(deployment_name, &functions_to_deploy)?;

        let existing_deployment = self.repository.get_deployment(deployment_name).await?;
        let previous_function_entities = match &existing_deployment {
            Some(deployment) => self.repository.get_functions(deployment.id).await?,
            None => Vec::new(),
        };

        let mut functions_to_create: Vec<FunctionToCreate> = Vec::new();
        for function_to_deploy in &functions_to_deploy {
            functions_to_create.push(
                self.deploy_function(
                    deployment_name,
                    &previous_function_entities,
                    function_to_deploy,
                    force,
                )
                .await?,
            );
        }

        // The record is only touched once everything is deployed, so a failed
        // deploy keeps the previous deployment intact.
        if let Some(deployment) = existing_deployment {
            self.undeploy_old_functions(
                &deployment,
                &previous_function_entities,
                &functions_to_deploy,
            )
            .await?;
            self.repository
                .replace_functions(deployment.id, &functions_to_create)
                .await?;
        } else {
            self.repository
                .insert_deployment_with_functions(&CreateDeploymentParams::new(
                    deployment_name.as_ref().clone(),
                    functions_to_create,
                ))
                .await?;
        }

        Ok(())
    }

    async fn undeploy_old_functions(
        &self,
        deployment: &DeploymentEntity,
        existing_function_entities: &[FunctionEntity],
        functions_to_deploy: &[FunctionToDeploy],
    ) -> Result<()> {
        for function_entity in existing_function_entities {
            let still_relevant = still_relevant(functions_to_deploy, function_entity);
            if !still_relevant {
                match function_entity {
                    FunctionEntity::Aws(aws) => {
                        self.aws_client
                            .delete_function(&AwsFunctionQualifier::new(
                                &deployment.name,
                                &aws.name,
                            ))
                            .await?;
                    }
                    FunctionEntity::Kn(kn) => {
                        self.kn_client
                            .delete_kn_function(&KnFunctionQualifier::new(
                                &deployment.name,
                                &kn.name,
                            ))
                            .await?;
                    }
                    FunctionEntity::Plugin(_plugin) => {}
                }
            }
        }
        Ok(())
    }

    async fn deploy_function(
        &self,
        deployment_name: &DeploymentName,
        previous_function_entities: &[FunctionEntity],
        function_to_deploy: &FunctionToDeploy,
        force: bool,
    ) -> Result<FunctionToCreate, DeployError> {
        let function_to_create = match function_to_deploy {
            FunctionToDeploy::Remote(remote_function_to_deploy) => {
                let function_entity = previous_function_entities
                    .iter()
                    .find(|e| e.raw().name == remote_function_to_deploy.name);
                let previous_hash = if force {
                    None
                } else {
                    function_entity.and_then(|e| e.raw().hash.clone())
                };
                match &remote_function_to_deploy.config {
                    FunctionConfig::Aws(aws_function_config) => {
                        let arn = crate::aws::deploy_aws_function(
                            remote_function_to_deploy,
                            aws_function_config,
                            previous_hash,
                            deployment_name,
                            self.aws_client.as_ref(),
                        )
                        .await?;
                        FunctionToCreate::Aws(AwsFunctionToCreate {
                            name: remote_function_to_deploy.name.clone(),
                            arn: arn.0,
                            hash: remote_function_to_deploy.hash.clone(),
                        })
                    }
                    FunctionConfig::Kn(kn_function_config) => {
                        let url = crate::kn::deploy_kn_function(
                            remote_function_to_deploy,
                            kn_function_config,
                            previous_hash,
                            deployment_name,
                            self.kn_client.as_ref(),
                        )
                        .await?;
                        FunctionToCreate::Kn(KnFunctionToCreate {
                            name: remote_function_to_deploy.name.clone(),
                            url: url.0,
                            hash: remote_function_to_deploy.hash.clone(),
                        })
                    }
                }
            }
            FunctionToDeploy::Plugin(plugin_function_to_deploy) => {
                deploy_plugin(plugin_function_to_deploy).await?
            }
        };
        Ok(function_to_create)
    }
}

fn still_relevant(
    functions_to_deploy: &[FunctionToDeploy],
    function_entity: &FunctionEntity,
) -> bool {
    functions_to_deploy.iter().any(|f| {
        if let FunctionToDeploy::Remote(remote_function_to_deploy) = f {
            match &remote_function_to_deploy.config {
                FunctionConfig::Aws(_aws_function_config) => {
                    return matches!(function_entity, FunctionEntity::Aws(_))
                        && f.name() == &function_entity.raw().name;
                }
                FunctionConfig::Kn(_kn_function_config) => {
                    // Knative service names are lowercased, so names that
                    // differ only in case map to the same service.
                    return matches!(function_entity, FunctionEntity::Kn(_))
                        && f.name()
                            .as_ref()
                            .eq_ignore_ascii_case(function_entity.raw().name.as_ref());
                }
            }
        }
        false
    })
}

async fn deploy_plugin(
    plugin_function_to_deploy: &PluginFunctionToDeploy,
) -> Result<FunctionToCreate, DeployError> {
    let function_to_create = FunctionToCreate::Plugin(PluginFunctionToCreate {
        name: plugin_function_to_deploy.name.clone(),
        file_name: plugin_function_to_deploy
            .path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
        blob: tokio::fs::read(&plugin_function_to_deploy.path).await?,
    });
    Ok(function_to_create)
}

fn extract_zip(zip_file: &File) -> Result<TempDir, DeployError> {
    let temp_deployment_dir = tempfile::tempdir()?;
    let mut zip_archive = ZipArchive::new(zip_file).context("Failed to open zip file")?;
    zip_archive
        .extract(temp_deployment_dir.path())
        .context("Failed to extract zip file")?;
    Ok(temp_deployment_dir)
}

// Knative service names are DNS labels, which allow no underscores.
fn validate_kn_names(
    deployment_name: &DeploymentName,
    functions_to_deploy: &[FunctionToDeploy],
) -> Result<(), DeployError> {
    for function_to_deploy in functions_to_deploy {
        let FunctionToDeploy::Remote(remote) = function_to_deploy else {
            continue;
        };
        if !matches!(remote.config, FunctionConfig::Kn(_)) {
            continue;
        }
        if remote.name.as_ref().contains('_') {
            return Err(DeployError::InvalidName(format!(
                "Knative function name {} must not contain underscores",
                remote.name
            )));
        }
        if deployment_name.as_ref().contains('_') {
            return Err(DeployError::InvalidName(format!(
                "Deployment name {deployment_name} must not contain underscores when deploying Knative functions"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detect::RemoteFunctionToDeploy;
    use florca_core::function::{AwsFunctionConfig, KnFunctionConfig, RawFunctionEntity};

    fn function(name: &str, config: FunctionConfig) -> FunctionToDeploy {
        FunctionToDeploy::Remote(RemoteFunctionToDeploy {
            name: name.into(),
            path: "unused".into(),
            hash: String::new(),
            config,
        })
    }

    fn kn_function(name: &str) -> FunctionToDeploy {
        function(
            name,
            FunctionConfig::Kn(KnFunctionConfig {
                runtime: "python".to_string(),
            }),
        )
    }

    fn kn_entity(name: &str) -> FunctionEntity {
        FunctionEntity::Kn(RawFunctionEntity {
            id: 1,
            deployment_id: 1,
            name: name.into(),
            kind: "kn".to_string(),
            location: String::new(),
            hash: None,
            blob: None,
        })
    }

    #[test]
    fn test_case_renamed_kn_function_stays_relevant() {
        let functions = vec![kn_function("fetchdata")];
        assert!(still_relevant(&functions, &kn_entity("fetchData")));
        assert!(!still_relevant(&functions, &kn_entity("other")));
    }

    #[test]
    fn test_underscores_are_rejected_for_kn_functions() {
        let valid = vec![kn_function("fetchData")];
        assert!(validate_kn_names(&"demo".into(), &valid).is_ok());
        assert!(validate_kn_names(&"my_demo".into(), &valid).is_err());

        let invalid = vec![kn_function("my_func")];
        assert!(validate_kn_names(&"demo".into(), &invalid).is_err());

        // AWS functions may keep their underscores
        let aws = vec![function(
            "my_func",
            FunctionConfig::Aws(AwsFunctionConfig {
                runtime: "nodejs24.x".to_string(),
                handler: "index.handler".to_string(),
                memory: 128,
                timeout: 3,
            }),
        )];
        assert!(validate_kn_names(&"my_demo".into(), &aws).is_ok());
    }
}
