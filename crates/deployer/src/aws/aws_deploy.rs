use crate::aws::aws_client::AwsClient;
use crate::aws::aws_qualifier::{Arn, AwsFunctionQualifier};
use crate::detect::RemoteFunctionToDeploy;
use anyhow::Result;
use florca_core::deployment::DeploymentName;
use florca_core::function::AwsFunctionConfig;
use std::path::Path;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use tracing::debug;
use tracing::info;

pub async fn deploy_aws_function(
    remote_function_to_deploy: &RemoteFunctionToDeploy,
    aws_function_config: &AwsFunctionConfig,
    previous_hash: Option<String>,
    deployment_name: &DeploymentName,
    aws_client: &dyn AwsClient,
) -> Result<Arn> {
    let implementation_path = Path::new(&remote_function_to_deploy.path).join("aws");
    let named_zip_file = zip_aws_function(&implementation_path)?;
    let zip_path = named_zip_file.path();
    let aws_function_qualifier =
        AwsFunctionQualifier::new(deployment_name, &remote_function_to_deploy.name);
    let existing_function = aws_client
        .find_deployed_function(&aws_function_qualifier)
        .await?;
    let hash = &remote_function_to_deploy.hash;
    if let Some(previous_hash) = &previous_hash {
        debug!(
            previous = previous_hash,
            new = hash,
            "Comparing hashes for {}",
            &remote_function_to_deploy.name
        );
        if previous_hash == hash
            && let Some(existing_function) = &existing_function
        {
            return Ok(existing_function.clone());
        }
    }
    info!("Deploying aws remote function {:?}", &implementation_path);
    let aws_function = if let Some(existing_function) = existing_function {
        aws_client
            .update_function(&aws_function_qualifier, aws_function_config, zip_path)
            .await?;
        existing_function.clone()
    } else {
        aws_client
            .create_function(&aws_function_qualifier, aws_function_config, zip_path)
            .await?
    };
    tokio::fs::remove_file(zip_path).await?;
    Ok(aws_function)
}

fn zip_aws_function(implementation_path: &PathBuf) -> Result<NamedTempFile> {
    let named_zip_file = tempfile::NamedTempFile::with_suffix(".zip")?;
    zip_extensions::zip_writer::zip_create_from_directory(
        &named_zip_file.path().to_path_buf(),
        implementation_path,
    )?;
    Ok(named_zip_file)
}
