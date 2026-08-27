use super::super::persist::ReviewStore;

pub(in crate::store) struct Persistence;

impl Persistence {
    pub(in crate::store) fn in_memory() -> Self {
        Self
    }
}

impl ReviewStore {
    pub(crate) fn save(&mut self) {
        let _ = &self.persistence;
    }
}
