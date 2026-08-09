//! One process-wide `ReviewStore` shared by every `RepoWindow` via a GPUI global; per-window copies would each rewrite `review_store.json` from their own snapshot, clobbering marks made in other windows.

use std::cell::RefCell;
use std::path::PathBuf;
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

pub fn install_from_path(cx: &mut App, path: PathBuf) {
    cx.set_global(ReviewStoreHandle(Rc::new(RefCell::new(
        jayjay_review::ReviewStore::load_from(path),
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
        if identity.is_empty() {
            return;
        }
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

    /// `cx: &App` (not `&Context<Self>`) so this stays callable from contexts that only hold an `&App`; `&Context<Self>` still coerces in at existing call sites.
    pub(super) fn review_file_context(&self, cx: &App) -> (bool, Option<String>) {
        let vm = self.vm.read(cx);
        let show_review = vm.shows_review_controls();
        let change_id = vm.selected_change().map(|c| c.change_id.id.clone());
        (show_review, change_id)
    }

    /// Every note surface (gutter dot, menu items, note rows, composer, badges) must gate through this: working-copy diff, outside compare mode, a real non-projected hunk with a review identity.
    pub fn review_notes_context(&self, hunk: &DiffHunk, cx: &App) -> Option<String> {
        let (show_review, change_id) = self.review_file_context(cx);
        if !show_review || hunk.projection.is_some() || hunk.review_identity.is_empty() {
            return None;
        }
        change_id
    }

    /// Working-copy change id when notes should be shown; `None` in compare mode or on a non-working-copy change, which triggers clearing rather than reconciling elsewhere.
    fn review_notes_change_id(&self, cx: &App) -> Option<String> {
        let vm = self.vm.read(cx);
        vm.shows_review_controls()
            .then(|| vm.selected_change().map(|c| c.change_id.id.clone()))
            .flatten()
    }

    /// Always `refresh_if_stale()` before reading the shared store, never a raw read on the long-lived global; returns raw notes with `include_resolved: true` so a resolved note still surfaces its dimmed dot.
    fn snapshot_review_notes(&self, change_id: &str) -> Vec<jayjay_review::NoteEntry> {
        let mut store = self.review_store.borrow_mut();
        store.refresh_if_stale();
        store.list_notes(change_id, true)
    }

    /// Callers must call this after mutating through `mutate()` so the reconciled `vm.review_notes` (and cached row list) reflects the write; always re-snapshots, unlike `sync_review_notes` below, which only does so when the sync key changed.
    pub fn refresh_review_notes(&mut self, cx: &mut Context<Self>) {
        let notes = match self.review_notes_change_id(cx) {
            Some(change_id) => {
                let notes = self.snapshot_review_notes(&change_id);
                self.diff.review_notes_sync_key =
                    Some((self.review_files_fingerprint(cx), notes.clone()));
                notes
            }
            None => {
                self.diff.review_notes_sync_key = None;
                Vec::new()
            }
        };
        self.vm.update(cx, |vm, cx| vm.load_review_notes(notes, cx));
    }

    /// Detects both note writes this process didn't make and diff refreshes that change file identities, since reconciliation depends on both; also how the first load for a newly-selected working-copy change happens, via `None != Some(key)`.
    pub fn sync_review_notes(&mut self, cx: &mut Context<Self>) {
        let Some(change_id) = self.review_notes_change_id(cx) else {
            // Gate just turned off: drop the last session's key too, or `vm.review_notes` (which `active_note_counts`/`stale_or_orphaned_notes` assume is already empty outside the notes session) would keep serving badges/banners for a change no longer shown.
            if self.diff.review_notes_sync_key.take().is_some() {
                self.vm
                    .update(cx, |vm, cx| vm.load_review_notes(Vec::new(), cx));
            }
            return;
        };
        let key = (
            self.review_files_fingerprint(cx),
            self.snapshot_review_notes(&change_id),
        );
        if self.diff.review_notes_sync_key.as_ref() != Some(&key) {
            let notes = key.1.clone();
            self.diff.review_notes_sync_key = Some(key);
            self.vm.update(cx, |vm, cx| vm.load_review_notes(notes, cx));
        }
    }

    /// Hash of every hunk's (path, review_identity): identities change exactly when file bytes change, which reconciliation output depends on besides the notes themselves.
    fn review_files_fingerprint(&self, cx: &App) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        if let Some(files) = self.vm.read(cx).files.as_ref() {
            for hunk in files.iter() {
                hunk.path.hash(&mut hasher);
                hunk.review_identity.hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    pub(crate) fn toggle_reviewed_for_selected_file(&mut self, cx: &mut Context<Self>) {
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
            let Some(hunk) = vm
                .selected_hunk()
                .filter(|hunk| !hunk.review_identity.is_empty())
            else {
                return;
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
            let files = self.vm.read(cx).files.clone();
            let row = files
                .map(|files| self.visible_indices(&files, change_id.as_deref(), show_review, cx))
                .and_then(|visible| visible.iter().position(|ix| *ix == next_ix))
                .unwrap_or(next_ix);
            self.scrolls
                .files
                .scroll_to_item(row, ScrollStrategy::Center);
            return;
        }
        cx.notify();
    }
}
