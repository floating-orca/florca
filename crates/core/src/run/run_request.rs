use crate::{deployment::DeploymentName, function::FunctionName};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunRequest {
    pub deployment_name: DeploymentName,
    pub entry_point: FunctionName,
    pub input: Value,
    pub params: Value,
}
