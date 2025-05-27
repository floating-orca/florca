use crate::{
    deployment::DeploymentName,
    function::FunctionName,
    invocation::{InvocationEntity, InvocationId},
    run::{RunEntity, RunId},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inspection {
    pub run_id: RunId,
    pub deployment_name: DeploymentName,
    pub entry_point: FunctionName,
    pub input: Value,
    pub output: Option<Value>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub root: Option<InspectionEntry>,
    pub run_status: RunStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RunStatus {
    Error,
    Running,
    Success,
}

impl Inspection {
    #[must_use]
    pub fn workflow_is_running(&self) -> bool {
        matches!(self.run_status, RunStatus::Running)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectionEntry {
    pub invocation_id: InvocationId,
    pub function_name: FunctionName,
    pub input: Value,
    pub params: Value,
    pub output: Option<Value>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub children: Vec<InspectionEntry>,
    pub next: Option<Box<InspectionEntry>>,
}

impl Inspection {
    #[must_use]
    pub fn new(run: RunEntity, root: Option<InspectionEntry>, status: RunStatus) -> Self {
        Inspection {
            run_id: run.id,
            deployment_name: run.deployment_name,
            entry_point: run.entry_point,
            input: run.input,
            output: run.output,
            start_time: run.start_time,
            end_time: run.end_time,
            root,
            run_status: status,
        }
    }
}

impl InspectionEntry {
    #[must_use]
    pub fn new(
        invocation: &InvocationEntity,
        children: Vec<InspectionEntry>,
        next: Option<Box<InspectionEntry>>,
    ) -> Self {
        Self {
            invocation_id: invocation.id,
            function_name: invocation.function_name.clone(),
            input: invocation.input.clone(),
            params: invocation.params.clone(),
            output: invocation.output.clone(),
            start_time: invocation.start_time,
            end_time: invocation.end_time,
            children,
            next,
        }
    }
}
