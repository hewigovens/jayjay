use std::collections::HashMap;

use jayjay_primitives::NoteEntry;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ReviewEntry {
    /// Caller-supplied content identity at mark time. Treated as opaque.
    pub(crate) identity: String,
    #[serde(default)]
    pub(crate) file_marked: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) hunks: Vec<u32>,
}

impl ReviewEntry {
    pub(crate) fn marked_file(identity: &str) -> Self {
        Self {
            identity: identity.to_string(),
            file_marked: true,
            hunks: vec![],
        }
    }

    pub(crate) fn marked_hunks(identity: &str, hunks: Vec<u32>) -> Self {
        Self {
            identity: identity.to_string(),
            file_marked: false,
            hunks,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct StoredReviews {
    #[serde(default, deserialize_with = "deserialize_reviewed")]
    pub(crate) reviewed: HashMap<String, ReviewEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) notes: Vec<StoredNote>,
    #[serde(flatten)]
    pub(crate) extra: serde_json::Map<String, serde_json::Value>,
}

/// Notes written by newer versions survive round trips through this build — unparseable entries are carried through save verbatim and unknown fields are re-merged on write — because the store file is shared by the app and the CLI, which can genuinely be at different versions.
#[derive(Debug, Clone)]
pub(crate) enum StoredNote {
    Parsed {
        note: Box<NoteEntry>,
        unknown_fields: serde_json::Map<String, serde_json::Value>,
    },
    Unknown(serde_json::Value),
}

impl StoredNote {
    pub(crate) fn new(note: NoteEntry) -> Self {
        StoredNote::Parsed {
            note: Box::new(note),
            unknown_fields: serde_json::Map::new(),
        }
    }

    pub(crate) fn parsed(&self) -> Option<&NoteEntry> {
        match self {
            StoredNote::Parsed { note, .. } => Some(note),
            StoredNote::Unknown(_) => None,
        }
    }

    pub(crate) fn parsed_mut(&mut self) -> Option<&mut NoteEntry> {
        match self {
            StoredNote::Parsed { note, .. } => Some(note),
            StoredNote::Unknown(_) => None,
        }
    }
}

impl Serialize for StoredNote {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            StoredNote::Parsed {
                note,
                unknown_fields,
            } => {
                let mut value = serde_json::to_value(note).map_err(serde::ser::Error::custom)?;
                if let serde_json::Value::Object(map) = &mut value {
                    for (key, field) in unknown_fields {
                        map.entry(key.clone()).or_insert_with(|| field.clone());
                    }
                }
                value.serialize(serializer)
            }
            StoredNote::Unknown(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for StoredNote {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let Ok(note) = NoteEntry::deserialize(&value) else {
            return Ok(StoredNote::Unknown(value));
        };
        let known = match serde_json::to_value(&note) {
            Ok(serde_json::Value::Object(map)) => map,
            _ => serde_json::Map::new(),
        };
        let unknown_fields = match value {
            serde_json::Value::Object(map) => map
                .into_iter()
                .filter(|(key, _)| !known.contains_key(key))
                .collect(),
            _ => serde_json::Map::new(),
        };
        Ok(StoredNote::Parsed {
            note: Box::new(note),
            unknown_fields,
        })
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

pub(crate) fn key(change_id: &str, path: &str) -> String {
    format!("{change_id}|{path}")
}
