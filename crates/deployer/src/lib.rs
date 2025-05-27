use anyhow::Result;
use aws::AwsClientImpl;
use deployer::Deployer;
use kn::KnClientImpl;
use repository::SqlxDeployerRepository;
use service::{DeployerService, DeployerServiceImpl};
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod aws;
pub mod deployer;
pub mod detect;
mod errors;
mod http;
pub mod kn;
mod pack;
pub mod repository;
pub mod service;

pub use http::serve;

#[derive(Debug)]
pub struct AppState {
    pub deployer_service: Arc<dyn DeployerService>,
}

pub async fn init() -> Result<Arc<RwLock<AppState>>> {
    let deployer_repository = Arc::new(SqlxDeployerRepository::setup().await?);
    let aws_client = Arc::new(AwsClientImpl::new().await);
    let kn_client = Arc::new(KnClientImpl::new());
    let deployer = Arc::new(Deployer::new(
        deployer_repository.clone(),
        aws_client.clone(),
        kn_client.clone(),
    ));
    let deployer_service = Arc::new(DeployerServiceImpl::new(deployer_repository, deployer));
    let state = AppState { deployer_service };
    Ok(Arc::new(RwLock::new(state)))
}
