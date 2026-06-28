use jayjay_primitives::NoteEntry;
use serde::{Deserialize, Deserializer, Serialize};

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
