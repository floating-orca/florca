use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "provider")]
#[derive(TS)]
pub enum FunctionConfig {
    Aws(AwsFunctionConfig),
    Kn(KnFunctionConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
pub struct AwsFunctionConfig {
    pub handler: String,
    pub runtime: String,
    pub memory: i32,
    pub timeout: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
pub struct KnFunctionConfig {
    pub runtime: String,
}
