use super::super::persist::ReviewStore;

pub(crate) struct Persistence;

impl Persistence {
    pub(crate) fn is_persistent(&self) -> bool {
        false
    }

    pub(crate) fn in_memory() -> Self {
        Self
    }
}

impl ReviewStore {
    pub(crate) fn begin_mutation(&mut self) -> std::io::Result<Option<()>> {
        Ok(None)
    }

    pub(crate) fn save_checked(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
