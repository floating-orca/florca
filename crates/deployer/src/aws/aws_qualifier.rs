use derive_more::{AsRef, Display, From};
use florca_core::{deployment::DeploymentName, function::FunctionName};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Arn(pub String);

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, From, AsRef, Display,
)]
pub struct AwsFunctionQualifier(String);

impl AwsFunctionQualifier {
    #[must_use]
    pub fn new(deployment_name: &DeploymentName, function_name: &FunctionName) -> Self {
        Self(format!("{deployment_name}-{function_name}"))
    }
}
