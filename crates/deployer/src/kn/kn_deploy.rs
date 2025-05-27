use crate::detect::RemoteFunctionToDeploy;
use crate::kn::KnClient;
use crate::kn::kn_qualifier::{KnFunctionQualifier, KnUrl};
use anyhow::Result;
use florca_core::{deployment::DeploymentName, function::KnFunctionConfig};
use std::path::Path;
use tracing::{debug, info};

pub async fn deploy_kn_function(
    remote_function_to_deploy: &RemoteFunctionToDeploy,
    kn_function_config: &KnFunctionConfig,
    previous_hash: Option<String>,
    deployment_name: &DeploymentName,
    kn_client: &dyn KnClient,
) -> Result<KnUrl> {
    let implementation_path = Path::new(&remote_function_to_deploy.path).join("kn");
    let kn_function_qualifier =
        KnFunctionQualifier::new(deployment_name, &remote_function_to_deploy.name);
    let existing_function = kn_client
        .find_deployed_kn_function(&kn_function_qualifier)
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
    info!("Deploying kn remote function {:?}", &implementation_path);
    let kn_function = kn_client
        .create_kn_function(
            &kn_function_qualifier,
            kn_function_config,
            &implementation_path,
        )
        .await?;
    Ok(kn_function)
}
