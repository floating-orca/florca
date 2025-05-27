use crate::run::RunId;
use serde::Deserialize;
use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AllOrRunId {
    All,
    RunId(RunId),
}

impl<'de> Deserialize<'de> for AllOrRunId {
    fn deserialize<D>(deserializer: D) -> Result<AllOrRunId, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        if s == "all" {
            Ok(AllOrRunId::All)
        } else {
            let run_id: RunId = s
                .parse()
                .map_err(|_| serde::de::Error::custom("invalid run id"))?;
            Ok(AllOrRunId::RunId(run_id))
        }
    }
}

impl Display for AllOrRunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllOrRunId::All => write!(f, "all"),
            AllOrRunId::RunId(run_id) => write!(f, "{run_id}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_all() {
        let json = r#""all""#;
        let result: AllOrRunId = serde_json::from_str(json).unwrap();
        assert_eq!(result, AllOrRunId::All);
    }

    #[test]
    fn test_deserialize_run_id() {
        let json = r#""123""#;
        let result: AllOrRunId = serde_json::from_str(json).unwrap();
        assert_eq!(result, AllOrRunId::RunId(RunId::new(123)));
    }

    #[test]
    fn test_display_all() {
        let all = AllOrRunId::All;
        assert_eq!(all.to_string(), "all");
    }

    #[test]
    fn test_display_run_id() {
        let run_id = AllOrRunId::RunId(RunId::new(456));
        assert_eq!(run_id.to_string(), "456");
    }
}
