use std::{
    fmt::{self, Display},
    str::FromStr,
};

use crate::{function::FunctionName, run::RunId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InvocationEntity {
    pub id: InvocationId,
    pub parent: Option<InvocationId>,
    pub predecessor: Option<InvocationId>,
    pub run_id: RunId,
    pub function_name: FunctionName,
    pub input: Value,
    pub params: Value,
    pub output: Option<Value>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(transparent, type_name = "SERIAL")]
#[derive(TS)]
pub struct InvocationId(i32);

impl InvocationId {
    /// # Panics
    ///
    /// Panics if the ID is less than 1
    #[must_use]
    pub fn new(id: i32) -> Self {
        assert!((id >= 1), "Invocation ID must be greater than 0");
        Self(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0} is not a valid invocation ID")]
pub struct ParseInvocationIdError(String);

impl TryFrom<&str> for InvocationId {
    type Error = ParseInvocationIdError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let value = i32::from_str(s).map_err(|_| ParseInvocationIdError(s.into()))?;
        if value < 1 {
            Err(ParseInvocationIdError(s.into()))
        } else {
            Ok(Self(value))
        }
    }
}

impl FromStr for InvocationId {
    type Err = ParseInvocationIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl Display for InvocationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
