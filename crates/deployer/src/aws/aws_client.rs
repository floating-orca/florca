use crate::aws::{aws_qualifier::Arn, aws_qualifier::AwsFunctionQualifier};
use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_lambda::{
    client::Waiters,
    error::ProvideErrorMetadata,
    primitives::Blob,
    types::{FunctionCode, Runtime},
};
use florca_core::function::AwsFunctionConfig;
use std::{env, fmt::Debug, path::Path, time::Duration};
use tracing::{info, warn};

#[async_trait::async_trait]
pub trait AwsClient: Debug + Send + Sync {
    async fn create_function(
        &self,
        aws_function_qualifier: &AwsFunctionQualifier,
        aws_function_config: &AwsFunctionConfig,
        zip_path: &Path,
    ) -> Result<Arn>;
    async fn find_deployed_function(
        &self,
        aws_function_qualifier: &AwsFunctionQualifier,
    ) -> Result<Option<Arn>>;
    async fn update_function(
        &self,
        aws_function_qualifier: &AwsFunctionQualifier,
        aws_function_config: &AwsFunctionConfig,
        zip_path: &Path,
    ) -> Result<()>;
    async fn delete_function(&self, aws_function_qualifier: &AwsFunctionQualifier) -> Result<()>;
}

#[derive(Debug)]
pub struct AwsClientImpl {
    client: aws_sdk_lambda::Client,
}

impl AwsClientImpl {
    pub async fn new() -> Self {
        let sdk_config = aws_config::defaults(BehaviorVersion::v2026_01_12())
            .load()
            .await;
        Self {
            client: aws_sdk_lambda::Client::new(&sdk_config),
        }
    }
}

#[async_trait::async_trait]
impl AwsClient for AwsClientImpl {
    async fn create_function(
        &self,
        aws_function_qualifier: &AwsFunctionQualifier,
        aws_function_config: &AwsFunctionConfig,
        zip_path: &Path,
    ) -> Result<Arn> {
        let role = env::var("AWS_ROLE")?;
        if role.is_empty() {
            anyhow::bail!("AWS_ROLE environment variable must not be empty");
        }
        let result = self
            .client
            .create_function()
            .function_name(aws_function_qualifier.as_ref())
            .runtime(Runtime::from(aws_function_config.runtime.as_str()))
            .role(&role)
            .handler(&aws_function_config.handler)
            .code(
                FunctionCode::builder()
                    .zip_file(Blob::new(tokio::fs::read(zip_path).await?))
                    .build(),
            )
            .memory_size(aws_function_config.memory)
            .timeout(aws_function_config.timeout)
            .send()
            .await?;
        self.client
            .wait_until_function_active_v2()
            .function_name(aws_function_qualifier.as_ref())
            .wait(Duration::from_secs(10))
            .await?;
        let arn = result.function_arn.unwrap();
        Ok(Arn(arn))
    }

    async fn find_deployed_function(
        &self,
        aws_function_qualifier: &AwsFunctionQualifier,
    ) -> Result<Option<Arn>> {
        let result = self
            .client
            .get_function()
            .function_name(aws_function_qualifier.as_ref())
            .send()
            .await;
        match result {
            Ok(output) => Ok(output
                .configuration()
                .and_then(|c| c.function_arn.clone())
                .map(Arn)),
            Err(e) if is_not_found(&e) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn update_function(
        &self,
        aws_function_qualifier: &AwsFunctionQualifier,
        aws_function_config: &AwsFunctionConfig,
        zip_path: &Path,
    ) -> Result<()> {
        self.client
            .update_function_configuration()
            .function_name(aws_function_qualifier.as_ref())
            .runtime(Runtime::from(aws_function_config.runtime.as_str()))
            .handler(&aws_function_config.handler)
            .memory_size(aws_function_config.memory)
            .timeout(aws_function_config.timeout)
            .send()
            .await?;
        self.client
            .wait_until_function_updated_v2()
            .function_name(aws_function_qualifier.as_ref())
            .wait(Duration::from_secs(10))
            .await?;
        self.client
            .update_function_code()
            .function_name(aws_function_qualifier.as_ref())
            .zip_file(Blob::new(tokio::fs::read(zip_path).await?))
            .send()
            .await?;
        self.client
            .wait_until_function_updated_v2()
            .function_name(aws_function_qualifier.as_ref())
            .wait(Duration::from_secs(10))
            .await?;
        Ok(())
    }

    async fn delete_function(&self, aws_function_qualifier: &AwsFunctionQualifier) -> Result<()> {
        let result = self
            .client
            .delete_function()
            .function_name(aws_function_qualifier.as_ref())
            .send()
            .await;

        match result {
            Ok(_) => info!(
                function = aws_function_qualifier.as_ref(),
                "Deleted AWS function"
            ),
            Err(e) if is_not_found(&e) => warn!(
                function = aws_function_qualifier.as_ref(),
                "Function not found"
            ),
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }
}

fn is_not_found(e: &impl ProvideErrorMetadata) -> bool {
    e.code() == Some("ResourceNotFoundException")
}
