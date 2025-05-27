use derive_more::AsRef;
use derive_more::Display;
use derive_more::From;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    From,
    AsRef,
    Display,
    sqlx::Type,
)]
#[sqlx(transparent, type_name = "TEXT")]
#[derive(TS)]
pub struct FunctionName(String);

impl From<&str> for FunctionName {
    fn from(name: &str) -> Self {
        Self(name.to_string())
    }
}
