use super::models::StoredReviews;
use super::persistence::Persistence;

pub trait IdSource: Send {
    fn next_id(&mut self) -> String;
}

pub struct UuidIdSource;

impl IdSource for UuidIdSource {
    fn next_id(&mut self) -> String {
        uuid::Uuid::new_v4().to_string()
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

    #[cfg(any(test, feature = "test-util"))]
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
}
