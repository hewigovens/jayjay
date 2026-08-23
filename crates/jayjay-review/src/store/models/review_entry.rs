use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ReviewEntry {
    /// Caller-supplied content identity at mark time. Treated as opaque.
    pub(crate) identity: String,
    #[serde(default)]
    pub(crate) file_marked: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) hunks: Vec<u32>,
    #[serde(flatten)]
    pub(crate) extra: serde_json::Map<String, serde_json::Value>,
}

impl ReviewEntry {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn marked_file(identity: &str) -> Self {
        Self {
            identity: identity.to_string(),
            file_marked: true,
            hunks: vec![],
            extra: serde_json::Map::new(),
        }
    }

    pub(crate) fn marked_hunks(identity: &str, hunks: Vec<u32>) -> Self {
        Self {
            identity: identity.to_string(),
            file_marked: false,
            hunks,
            extra: serde_json::Map::new(),
        }
    }
}

pub(crate) fn key(change_id: &str, path: &str) -> String {
    format!("{change_id}|{path}")
}
