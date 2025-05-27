use crate::function::{FunctionName, RawFunctionEntity};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct LookupEntry {
    pub name: FunctionName,
    pub kind: LookupEntryKind,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
pub enum LookupEntryKind {
    Aws,
    Kn,
    Plugin,
}

impl From<&RawFunctionEntity> for LookupEntry {
    fn from(entity: &RawFunctionEntity) -> Self {
        Self {
            name: entity.name.clone(),
            kind: match entity.kind.as_str() {
                "aws" => LookupEntryKind::Aws,
                "kn" => LookupEntryKind::Kn,
                "plugin" => LookupEntryKind::Plugin,
                _ => panic!("Unknown function type: {}", entity.kind),
            },
            location: entity.location.clone(),
        }
    }
}
