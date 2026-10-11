use std::cell::RefCell;
use std::path::Path;

use gpui::{App, Global};
use jayjay_core::RecentRevsetStore;

struct RecentRevsetsHandle(RefCell<RecentRevsetStore>);

impl Global for RecentRevsetsHandle {}

fn store(cx: &mut App) -> &RefCell<RecentRevsetStore> {
    if !cx.has_global::<RecentRevsetsHandle>() {
        cx.set_global(RecentRevsetsHandle(RefCell::new(RecentRevsetStore::load())));
    }
    &cx.global::<RecentRevsetsHandle>().0
}

pub(crate) fn recent(cx: &mut App, repository: &Path) -> Vec<String> {
    store(cx).borrow_mut().recent(repository)
}

pub(crate) fn remember(cx: &mut App, repository: &Path, revset: &str) -> Vec<String> {
    store(cx).borrow_mut().remember(repository, revset)
}

pub(crate) fn forget(cx: &mut App, repository: &Path, revset: &str) -> Vec<String> {
    store(cx).borrow_mut().forget(repository, revset)
}

pub fn install_in_memory(cx: &mut App) {
    cx.set_global(RecentRevsetsHandle(RefCell::new(
        RecentRevsetStore::in_memory(),
    )));
}
