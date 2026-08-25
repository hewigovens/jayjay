use jayjay_primitives::ReviewGroupState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StoredReviewGroup {
    pub(crate) digest: String,
    #[serde(default)]
    pub(crate) state: ReviewGroupState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum ReviewEntryState {
    File,
    Hunks {
        indices: Vec<u32>,
    },
    Groups {
        algorithm_version: u32,
        #[serde(default)]
        groups: Vec<StoredReviewGroup>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        removed_reviewed: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ReviewEntry {
    /// Caller-supplied content identity at mark time. Treated as opaque.
    pub(crate) identity: String,
    pub(crate) state: ReviewEntryState,
    /// Entry fields written by a newer binary; carried through saves so an older binary cannot strip them.
    #[serde(flatten)]
    pub(crate) extra: serde_json::Map<String, serde_json::Value>,
}

impl ReviewEntry {
    pub(crate) fn new(identity: &str, state: ReviewEntryState) -> Self {
        Self {
            identity: identity.to_string(),
            state,
            extra: serde_json::Map::new(),
        }
    }

    pub(crate) fn file(identity: &str) -> Self {
        Self::new(identity, ReviewEntryState::File)
    }

    pub(crate) fn hunks(identity: &str, mut indices: Vec<u32>) -> Self {
        indices.sort_unstable();
        indices.dedup();
        Self::new(identity, ReviewEntryState::Hunks { indices })
    }

    pub(crate) fn with_extra(mut self, extra: serde_json::Map<String, serde_json::Value>) -> Self {
        self.extra = extra;
        self
    }
}

pub(crate) fn key(change_id: &str, path: &str) -> String {
    format!("{change_id}|{path}")
}
