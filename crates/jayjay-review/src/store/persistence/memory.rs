use super::super::persist::ReviewStore;

pub(crate) struct Persistence;

impl Persistence {
    pub(crate) fn in_memory() -> Self {
        Self
    }
}

impl ReviewStore {
    pub(crate) fn save_checked(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    pub(crate) fn save(&mut self) {
        let _ = &self.persistence;
    }
}
