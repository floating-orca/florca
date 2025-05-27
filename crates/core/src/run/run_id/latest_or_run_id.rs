use crate::run::RunId;
use serde::Deserialize;
use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LatestOrRunId {
    Latest,
    RunId(RunId),
}

impl<'de> Deserialize<'de> for LatestOrRunId {
    fn deserialize<D>(deserializer: D) -> Result<LatestOrRunId, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        if s == "latest" {
            Ok(LatestOrRunId::Latest)
        } else {
            let run_id: RunId = s
                .parse()
                .map_err(|_| serde::de::Error::custom("invalid run id"))?;
            Ok(LatestOrRunId::RunId(run_id))
        }
    }
}

impl Display for LatestOrRunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LatestOrRunId::Latest => write!(f, "latest"),
            LatestOrRunId::RunId(run_id) => write!(f, "{run_id}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_latest() {
        let json = r#""latest""#;
        let result: LatestOrRunId = serde_json::from_str(json).unwrap();
        assert_eq!(result, LatestOrRunId::Latest);
    }

    #[test]
    fn test_deserialize_run_id() {
        let json = r#""123""#;
        let result: LatestOrRunId = serde_json::from_str(json).unwrap();
        assert_eq!(result, LatestOrRunId::RunId(RunId::new(123)));
    }

    #[test]
    fn test_display_latest() {
        let latest = LatestOrRunId::Latest;
        assert_eq!(latest.to_string(), "latest");
    }

    #[test]
    fn test_display_run_id() {
        let run_id = LatestOrRunId::RunId(RunId::new(456));
        assert_eq!(run_id.to_string(), "456");
    }
}
