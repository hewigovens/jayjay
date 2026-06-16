//! One process-wide `ReviewStore` shared by every `RepoWindow` via a GPUI global.
//! Per-window copies would each rewrite `review_store.json` from their own snapshot,
//! clobbering marks made in other windows.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{App, Global};

/// Shared in-memory review state. Cloning the handle shares the same store.
pub type SharedReviewStore = Rc<RefCell<jayjay_core::review::ReviewStore>>;

struct ReviewStoreHandle(SharedReviewStore);

impl Global for ReviewStoreHandle {}

impl Default for ReviewStoreHandle {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(
            jayjay_core::review::ReviewStore::load(),
        )))
    }
}

/// The single store for this process, loaded from disk on first use.
pub fn shared(cx: &mut App) -> SharedReviewStore {
    cx.default_global::<ReviewStoreHandle>().0.clone()
}

/// Install a non-persisting in-memory store so tests never touch the real `review_store.json`.
pub fn install_in_memory(cx: &mut App) {
    cx.set_global(ReviewStoreHandle(Rc::new(RefCell::new(
        jayjay_core::review::ReviewStore::in_memory(),
    ))));
}
