use crate::aws::aws_qualifier::AwsFunctionQualifier;
use crate::deployer::Deployer;
use crate::kn::kn_qualifier::KnFunctionQualifier;
use crate::{
    errors::{DeleteDeploymentError, DeployError, FetchDeploymentError, ListDeploymentsError},
    repository::DeployerRepository,
};
use florca_core::{deployment::DeploymentName, function::FunctionEntity};
use std::{fmt::Debug, sync::Arc};
use tokio::fs::File;

#[async_trait::async_trait]
pub trait DeployerService: Debug + Send + Sync {
    async fn list_deployments(&self) -> Result<Vec<DeploymentName>, ListDeploymentsError>;
    async fn deploy(
        &self,
        name: &DeploymentName,
        bytes: &[u8],
        force: bool,
    ) -> Result<(), DeployError>;
    async fn fetch_deployment(&self, name: &DeploymentName) -> Result<File, FetchDeploymentError>;
    async fn delete_deployment(&self, name: &DeploymentName) -> Result<(), DeleteDeploymentError>;
}

#[derive(Debug)]
pub struct DeployerServiceImpl {
    repository: Arc<dyn DeployerRepository>,
    deployer: Arc<Deployer>,
}

impl DeployerServiceImpl {
    pub fn new(repository: Arc<dyn DeployerRepository>, deployer: Arc<Deployer>) -> Self {
        DeployerServiceImpl {
            repository,
            deployer,
        }
    }
}

#[async_trait::async_trait]
impl DeployerService for DeployerServiceImpl {
    async fn list_deployments(&self) -> Result<Vec<DeploymentName>, ListDeploymentsError> {
        let deployments = self
            .repository
            .get_deployments()
            .await?
            .into_iter()
            .map(|d| d.name)
            .collect();
        Ok(deployments)
    }

    async fn deploy(
        &self,
        deployment_name: &DeploymentName,
        bytes: &[u8],
        force: bool,
    ) -> Result<(), DeployError> {
        self.deployer.deploy(bytes, deployment_name, force).await
    }

    async fn fetch_deployment(&self, name: &DeploymentName) -> Result<File, FetchDeploymentError> {
        let Some(deployment) = self.repository.get_deployment(name).await? else {
            return Err(FetchDeploymentError::NotFound(name.clone()));
        };
        let functions = self.repository.get_functions(deployment.id).await?;
        Ok(crate::pack::pack_deployment(functions).await?)
    }

    async fn delete_deployment(&self, name: &DeploymentName) -> Result<(), DeleteDeploymentError> {
        let Some(deployment) = self.repository.get_deployment(name).await? else {
            return Err(DeleteDeploymentError::NotFound(name.clone()));
        };
        let function_entities = self.repository.get_functions(deployment.id).await?;
        self.repository.delete_deployment(name).await?;
        for function_entity in function_entities {
            match function_entity {
                FunctionEntity::Aws(aws) => {
                    self.deployer
                        .aws_client
                        .delete_function(&AwsFunctionQualifier::new(&deployment.name, &aws.name))
                        .await?;
                }
                FunctionEntity::Kn(kn) => {
                    self.deployer
                        .kn_client
                        .delete_kn_function(&KnFunctionQualifier::new(&deployment.name, &kn.name))
                        .await?;
                }
                FunctionEntity::Plugin(_plugin) => {}
            }
        }
        Ok(())
    }
}
