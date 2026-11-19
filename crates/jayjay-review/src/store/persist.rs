use jayjay_primitives::ReviewStoreSummary;

use super::models::StoredReviews;
use super::persistence::Persistence;

pub trait IdSource: Send {
    fn next_id(&mut self) -> String;
}

pub struct UuidIdSource;

impl IdSource for UuidIdSource {
    fn next_id(&mut self) -> String {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).expect("system random source");
        // Version 4 and variant bits keep new ids in the UUID shape that stored notes already use.
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        let hex = hex::encode(bytes);
        format!(
            "{}-{}-{}-{}-{}",
            &hex[..8],
            &hex[8..12],
            &hex[12..16],
            &hex[16..20],
            &hex[20..]
        )
    }
}

pub struct ReviewStore {
    pub(crate) state: StoredReviews,
    pub(super) persistence: Persistence,
    pub(crate) id_source: Box<dyn IdSource>,
}

impl ReviewStore {
    pub fn in_memory() -> Self {
        Self::from_state(StoredReviews::default())
    }

    pub fn in_memory_from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json).map(Self::from_state)
    }

    pub fn snapshot_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self.state)
    }

    #[cfg(test)]
    pub fn in_memory_with_ids(id_source: Box<dyn IdSource>) -> Self {
        Self {
            state: StoredReviews::default(),
            persistence: Persistence::in_memory(),
            id_source,
        }
    }

    pub(crate) fn from_state(state: StoredReviews) -> Self {
        Self {
            state,
            persistence: Persistence::in_memory(),
            id_source: Box::new(UuidIdSource),
        }
    }

    pub fn summary(&self) -> ReviewStoreSummary {
        ReviewStoreSummary {
            marks: self.state.reviewed.len() as u32,
            notes: self.state.notes.len() as u32,
        }
    }

    /// Drops every mark and note for every repository; unknown top-level fields stay for other versions.
    pub fn clear_all(&mut self) {
        self.mutate(|store| {
            store.state.reviewed.clear();
            store.state.notes.clear();
            store.state.review_modes.clear();
        })
    }
}
