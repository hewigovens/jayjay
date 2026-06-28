//! One process-wide `ReviewStore` shared by every `RepoWindow` via a GPUI global; per-window copies would each rewrite `review_store.json` from their own snapshot, clobbering marks made in other windows.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{App, Context, Global, ScrollStrategy};
use jayjay_core::DiffHunk;

use super::RepoWindow;

pub type SharedReviewStore = Rc<RefCell<jayjay_review::ReviewStore>>;

struct ReviewStoreHandle(SharedReviewStore);

impl Global for ReviewStoreHandle {}

impl Default for ReviewStoreHandle {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(jayjay_review::ReviewStore::load())))
    }
}

pub fn shared(cx: &mut App) -> SharedReviewStore {
    cx.default_global::<ReviewStoreHandle>().0.clone()
}

/// Mutate against fresh disk state: the store lives for the whole process, so saving its startup snapshot would clobber marks and notes the CLI or the SwiftUI shell persisted since load.
pub fn mutate<R>(
    store: &SharedReviewStore,
    f: impl FnOnce(&mut jayjay_review::ReviewStore) -> R,
) -> R {
    let mut store = store.borrow_mut();
    store.refresh_from_disk();
    f(&mut store)
}

/// Install a non-persisting in-memory store so tests never touch the real `review_store.json`.
pub fn install_in_memory(cx: &mut App) {
    cx.set_global(ReviewStoreHandle(Rc::new(RefCell::new(
        jayjay_review::ReviewStore::in_memory(),
    ))));
}

impl RepoWindow {
    pub fn toggle_reviewed(
        &mut self,
        change_id: String,
        path: String,
        identity: String,
        cx: &mut Context<Self>,
    ) {
        mutate(&self.review_store, |store| {
            store.toggle(&change_id, &path, &identity);
        });
        cx.notify();
    }

    pub fn is_reviewed(&self, change_id: &str, path: &str, identity: &str) -> bool {
        let mut store = self.review_store.borrow_mut();
        // Render-path reads must notice marks the CLI or SwiftUI shell wrote while this window was open; only mutations refresh otherwise.
        store.refresh_if_stale();
        store.is_reviewed(change_id, path, identity)
    }

    pub(super) fn review_file_context(&self, cx: &Context<Self>) -> (bool, Option<String>) {
        let vm = self.vm.read(cx);
        let show_review = vm
            .selected_change()
            .map(|change| change.is_working_copy)
            .unwrap_or(false)
            && vm.compare.is_none();
        let change_id = vm.selected_change().map(|c| c.change_id.id.clone());
        (show_review, change_id)
    }

    pub(crate) fn visible_file_indices(
        &self,
        files: &[DiffHunk],
        change_id: Option<&str>,
        show_review: bool,
    ) -> Vec<usize> {
        let hide_reviewed = show_review && self.file_column.hide_reviewed;
        let Some(change_id) = change_id.filter(|_| hide_reviewed) else {
            return (0..files.len()).collect();
        };
        files
            .iter()
            .enumerate()
            .filter(|(_, hunk)| !self.is_reviewed(change_id, &hunk.path, &hunk.review_identity))
            .map(|(ix, _)| ix)
            .collect()
    }

    pub fn toggle_hide_reviewed_files(&mut self, cx: &mut Context<Self>) {
        self.file_column.hide_reviewed ^= true;
        if self.file_column.hide_reviewed {
            let (show_review, change_id) = self.review_file_context(cx);
            let (selected, visible) = {
                let vm = self.vm.read(cx);
                let visible = vm
                    .files
                    .as_ref()
                    .map(|files| {
                        self.visible_file_indices(files, change_id.as_deref(), show_review)
                    })
                    .unwrap_or_default();
                (vm.selected_file_ix, visible)
            };
            if selected.is_some_and(|ix| !visible.contains(&ix))
                && let Some(next) = visible.first().copied()
            {
                self.select_file(next, cx);
                self.scrolls.files.scroll_to_item(0, ScrollStrategy::Top);
                return;
            }
        }
        cx.notify();
    }

    pub fn toggle_reviewed_for_selected_file(&mut self, cx: &mut Context<Self>) {
        let (change_id, path, identity, files) = {
            let vm = self.vm.read(cx);
            if vm.compare.is_some() {
                return;
            }
            let change = match vm.selected_change() {
                Some(c) if c.is_working_copy => c,
                _ => return,
            };
            let change_id = change.change_id.clone();
            let hunk = match vm.selected_hunk() {
                Some(h) => h,
                None => return,
            };
            let path = hunk.path.clone();
            let identity = hunk.review_identity.clone();
            let files: Vec<(String, String)> = vm
                .files
                .as_ref()
                .map(|f| {
                    f.iter()
                        .map(|h| (h.path.clone(), h.review_identity.clone()))
                        .collect()
                })
                .unwrap_or_default();
            (change_id, path, identity, files)
        };
        let next_ix = mutate(&self.review_store, |store| {
            store.toggle(&change_id, &path, &identity);
            store
                .is_reviewed(&change_id, &path, &identity)
                .then(|| {
                    files
                        .iter()
                        .position(|(p, id)| !store.is_reviewed(&change_id, p, id))
                })
                .flatten()
        });
        if let Some(next_ix) = next_ix {
            self.select_file(next_ix, cx);
            let (show_review, change_id) = self.review_file_context(cx);
            let row = {
                let vm = self.vm.read(cx);
                vm.files
                    .as_ref()
                    .map(|files| {
                        self.visible_file_indices(files, change_id.as_deref(), show_review)
                    })
                    .and_then(|visible| visible.iter().position(|ix| *ix == next_ix))
                    .unwrap_or(next_ix)
            };
            self.scrolls
                .files
                .scroll_to_item(row, ScrollStrategy::Center);
            return;
        }
        cx.notify();
    }
}
