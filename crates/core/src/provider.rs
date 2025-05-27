use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(ValueEnum)]
pub enum Provider {
    Aws,
    Kn,
}

impl Provider {
    #[must_use]
    pub fn code(&self) -> &str {
        match self {
            Self::Aws => "aws",
            Self::Kn => "kn",
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Aws => "AWS",
            Self::Kn => "Knative",
        }
    }
}

impl From<&str> for Provider {
    fn from(s: &str) -> Self {
        match s {
            "aws" => Self::Aws,
            "kn" => Self::Kn,
            _ => panic!("Unknown provider: {s}"),
        }
    }
}
