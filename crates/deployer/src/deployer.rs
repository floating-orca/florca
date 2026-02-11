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

        let mut previous_function_entities: Vec<FunctionEntity> = Vec::new();
        if let Some(deployment) = self.repository.get_deployment(deployment_name).await? {
            let existing_function_entities = self.repository.get_functions(deployment.id).await?;
            self.repository.delete_deployment(deployment_name).await?;
            self.undeploy_old_functions(
                &deployment,
                &existing_function_entities,
                &functions_to_deploy,
            )
            .await?;
            previous_function_entities = existing_function_entities;
        }

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

        self.repository
            .insert_deployment_with_functions(&CreateDeploymentParams::new(
                deployment_name.as_ref().clone(),
                functions_to_create,
            ))
            .await?;

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
                    return matches!(function_entity, FunctionEntity::Kn(_))
                        && f.name() == &function_entity.raw().name;
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
