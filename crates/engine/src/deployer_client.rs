use crate::error::RunWorkflowError;
use anyhow::{Context, Result};
use async_trait::async_trait;
use florca_core::http::DeployerUrl;
use florca_core::{deployment::DeploymentName, http::RequestBuilderExt};
use reqwest::{Client, StatusCode};
use std::fmt::Debug;

#[async_trait]
pub trait DeployerClient: Debug + Send + Sync {
    async fn fetch_deployment_zip(
        &self,
        deployment_name: &DeploymentName,
    ) -> Result<Vec<u8>, RunWorkflowError>;
}

#[derive(Debug)]
pub struct DeployerClientImpl;

#[async_trait]
impl DeployerClient for DeployerClientImpl {
    async fn fetch_deployment_zip(
        &self,
        deployment_name: &DeploymentName,
    ) -> Result<Vec<u8>, RunWorkflowError> {
        let url = DeployerUrl::path(&[deployment_name.as_ref()]);
        let response = Client::new()
            .get(url)
            .with_basic_auth_from_env()
            .send()
            .await
            .with_context(|| "Failed to send request")?;
        match response.error_for_status() {
            Ok(response) => {
                let bytes = response
                    .bytes()
                    .await
                    .with_context(|| "Failed to read response")?;
                Ok(bytes.to_vec())
            }
            Err(e) if e.status() == Some(StatusCode::NOT_FOUND) => Err(
                RunWorkflowError::DeploymentNotFound(deployment_name.to_string()),
            ),
            Err(e) => Err(e).with_context(|| "Failed to fetch deployment")?,
        }
    }
}
