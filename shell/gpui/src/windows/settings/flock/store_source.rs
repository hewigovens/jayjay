use std::path::PathBuf;

use gpui::{App, Global};
use jayjay_core::flock::FlockStore;

#[derive(Clone, Default)]
pub(super) struct FlockStoreSource(Option<PathBuf>);

impl Global for FlockStoreSource {}

impl FlockStoreSource {
    pub(super) fn current(cx: &App) -> Self {
        cx.try_global::<Self>().cloned().unwrap_or_default()
    }

    pub(super) fn open(&self) -> FlockStore {
        self.0
            .as_deref()
            .map_or_else(FlockStore::load, FlockStore::at)
    }
}

pub fn install_flock_store(cx: &mut App, path: PathBuf) {
    cx.set_global(FlockStoreSource(Some(path)));
}
