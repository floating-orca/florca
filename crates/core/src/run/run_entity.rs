use crate::{deployment::DeploymentName, function::FunctionName, run::RunId};
use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RunEntity {
    pub id: RunId,
    pub deployment_name: DeploymentName,
    pub entry_point: FunctionName,
    pub input: Value,
    pub output: Option<Value>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub success: Option<bool>,
}
