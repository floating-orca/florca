use derive_more::{AsRef, Display, From};
use florca_core::{deployment::DeploymentName, function::FunctionName};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct KnUrl(pub String);

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, From, AsRef, Display,
)]
pub struct KnFunctionQualifier(String);

impl KnFunctionQualifier {
    #[must_use]
    pub fn new(deployment_name: &DeploymentName, function_name: &FunctionName) -> Self {
        let deployment_name = deployment_name.as_ref().to_lowercase();
        let function_name = function_name.as_ref().to_lowercase();
        Self(format!("{deployment_name}-{function_name}"))
    }
}
