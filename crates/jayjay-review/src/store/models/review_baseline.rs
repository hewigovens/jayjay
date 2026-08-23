use jayjay_primitives::ReviewGroupState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StoredBaselineGroup {
    pub(crate) digest: String,
    #[serde(default)]
    pub(crate) state: ReviewGroupState,
    #[serde(flatten)]
    pub(crate) extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ReviewBaseline {
    #[serde(default = "default_schema_version")]
    pub(crate) schema_version: u32,
    pub(crate) algorithm_version: u32,
    pub(crate) identity: String,
    #[serde(default)]
    pub(crate) groups: Vec<StoredBaselineGroup>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) removed_reviewed: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(crate) mirror_digest: String,
    #[serde(flatten)]
    pub(crate) extra: serde_json::Map<String, serde_json::Value>,
}

pub(crate) const BASELINE_SCHEMA_VERSION: u32 = 1;

fn default_schema_version() -> u32 {
    BASELINE_SCHEMA_VERSION
}
