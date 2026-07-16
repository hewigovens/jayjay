use std::cell::RefCell;
use std::path::Path;

use gpui::{App, BorrowAppContext, Global};
use jayjay_core::repositories::Store;

pub struct StoreHandle(RefCell<Store>);

impl Global for StoreHandle {}

impl Default for StoreHandle {
    fn default() -> Self {
        Self(RefCell::new(Store::load()))
    }
}

pub fn install(cx: &mut App) {
    cx.set_global(StoreHandle::default());
}

pub fn ensure(cx: &mut App) {
    if !cx.has_global::<StoreHandle>() {
        cx.set_global(StoreHandle::default());
    }
}

pub fn current(cx: &mut App) -> Vec<String> {
    ensure(cx);
    cx.global::<StoreHandle>().0.borrow_mut().repositories()
}

pub fn set_pinned(cx: &mut App, path: &Path, pinned: bool) {
    ensure(cx);
    cx.update_global::<StoreHandle, _>(|handle, _| {
        handle.0.borrow_mut().set_pinned(path, pinned);
    });
}

pub fn install_in_memory(cx: &mut App) {
    cx.set_global(StoreHandle(RefCell::new(Store::in_memory())));
}

pub(crate) fn repository_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(path)
        .to_owned()
}
