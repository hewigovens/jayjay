use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

use super::{ReviewBaseline, ReviewEntry, StoredNote};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct StoredReviews {
    #[serde(default, deserialize_with = "deserialize_reviewed")]
    pub(crate) reviewed: HashMap<String, ReviewEntry>,
    #[serde(
        default,
        skip_serializing_if = "HashMap::is_empty",
        deserialize_with = "deserialize_baselines"
    )]
    pub(crate) review_baselines: HashMap<String, ReviewBaseline>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) notes: Vec<StoredNote>,
    #[serde(flatten)]
    pub(crate) extra: serde_json::Map<String, serde_json::Value>,
}

impl StoredReviews {
    pub(crate) fn prune_orphan_baselines(&mut self) {
        self.review_baselines
            .retain(|key, _| self.reviewed.contains_key(key));
    }
}

// Drop unrecognized entry shapes (legacy mtime numbers, old hash-keyed entries) on load.
fn deserialize_reviewed<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<HashMap<String, ReviewEntry>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Loaded {
        Entry(ReviewEntry),
        #[allow(dead_code)]
        Unknown(serde_json::Value),
    }
    let raw: HashMap<String, Loaded> = HashMap::deserialize(d)?;
    Ok(raw
        .into_iter()
        .filter_map(|(k, v)| match v {
            Loaded::Entry(e) => Some((k, e)),
            Loaded::Unknown(_) => None,
        })
        .collect())
}

fn deserialize_baselines<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<HashMap<String, ReviewBaseline>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Loaded {
        Baseline(ReviewBaseline),
        #[allow(dead_code)]
        Unknown(serde_json::Value),
    }
    let raw: HashMap<String, Loaded> = HashMap::deserialize(d)?;
    Ok(raw
        .into_iter()
        .filter_map(|(k, v)| match v {
            Loaded::Baseline(baseline) => Some((k, baseline)),
            Loaded::Unknown(_) => None,
        })
        .collect())
}
