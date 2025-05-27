use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::str::FromStr;
use ts_rs::TS;

mod all_or_run_id;
mod latest_or_run_id;

pub use all_or_run_id::AllOrRunId;
pub use latest_or_run_id::LatestOrRunId;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(transparent, type_name = "SERIAL")]
#[derive(TS)]
pub struct RunId(i32);

impl RunId {
    /// # Panics
    ///
    /// Panics if the ID is less than 1
    #[must_use]
    pub fn new(id: i32) -> Self {
        assert!((id >= 1), "Run ID must be greater than 0");
        Self(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0} is not a valid run ID")]
pub struct ParseRunIdError(String);

impl TryFrom<&str> for RunId {
    type Error = ParseRunIdError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let value = i32::from_str(s).map_err(|_| ParseRunIdError(s.into()))?;
        if value < 1 {
            Err(ParseRunIdError(s.into()))
        } else {
            Ok(Self(value))
        }
    }
}

impl FromStr for RunId {
    type Err = ParseRunIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl Display for RunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_id_new() {
        let run_id = RunId::new(1);
        assert_eq!(run_id.0, 1);
    }

    #[test]
    #[should_panic(expected = "Run ID must be greater than 0")]
    fn test_run_id_new_zero() {
        let _ = RunId::new(0);
    }

    #[test]
    fn test_run_id_from_str() {
        let run_id: RunId = "42".parse().unwrap();
        assert_eq!(run_id.0, 42);
    }

    #[test]
    fn test_run_id_from_str_invalid() {
        let result: Result<RunId, _> = "invalid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_run_id_from_str_zero() {
        let result: Result<RunId, _> = "0".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_run_id_display() {
        let run_id = RunId::new(123);
        assert_eq!(run_id.to_string(), "123");
    }
}
