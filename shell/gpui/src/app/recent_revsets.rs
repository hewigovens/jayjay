use std::cell::RefCell;

use gpui::{App, Global};
use jayjay_core::RecentRevsetStore;

struct RecentRevsetsHandle(RefCell<RecentRevsetStore>);

impl Global for RecentRevsetsHandle {}

pub(crate) fn update<R>(cx: &mut App, f: impl FnOnce(&mut RecentRevsetStore) -> R) -> R {
    if !cx.has_global::<RecentRevsetsHandle>() {
        cx.set_global(RecentRevsetsHandle(RefCell::new(RecentRevsetStore::load())));
    }
    f(&mut cx.global::<RecentRevsetsHandle>().0.borrow_mut())
}

pub fn install_in_memory(cx: &mut App) {
    cx.set_global(RecentRevsetsHandle(RefCell::new(
        RecentRevsetStore::in_memory(),
    )));
}
