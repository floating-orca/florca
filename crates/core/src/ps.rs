use crate::{deployment::DeploymentName, run::RunId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningWorkflow {
    pub run_id: RunId,
    pub name: DeploymentName,
}
