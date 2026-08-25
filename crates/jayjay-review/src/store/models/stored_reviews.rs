use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

use super::{ReviewEntry, StoredNote};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct StoredReviews {
    #[serde(default, deserialize_with = "deserialize_reviewed")]
    pub(crate) reviewed: HashMap<String, ReviewEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) notes: Vec<StoredNote>,
    #[serde(flatten)]
    pub(crate) extra: serde_json::Map<String, serde_json::Value>,
}

// Pre-tag `file_marked`/`hunks` entries migrate to file/hunk states; anything else unreadable is dropped without affecting notes.
fn deserialize_reviewed<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<HashMap<String, ReviewEntry>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Loaded {
        Entry(ReviewEntry),
        Legacy(LegacyEntry),
        Unknown(serde::de::IgnoredAny),
    }
    let raw: HashMap<String, Loaded> = HashMap::deserialize(d)?;
    Ok(raw
        .into_iter()
        .filter_map(|(k, v)| match v {
            Loaded::Entry(entry) => Some((k, entry)),
            Loaded::Legacy(legacy) => legacy.migrate().map(|entry| (k, entry)),
            Loaded::Unknown(_) => None,
        })
        .collect())
}

#[derive(Deserialize)]
struct LegacyEntry {
    identity: String,
    #[serde(default)]
    file_marked: bool,
    #[serde(default)]
    hunks: Vec<u32>,
}

impl LegacyEntry {
    fn migrate(self) -> Option<ReviewEntry> {
        if self.file_marked {
            Some(ReviewEntry::file(&self.identity))
        } else if self.hunks.is_empty() {
            None
        } else {
            Some(ReviewEntry::hunks(&self.identity, self.hunks))
        }
    }
}
