//! One process-wide `ReviewStore`: per-window copies would each rewrite `review_store.json` from their own snapshot, clobbering other windows' marks.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{App, Context, Global, ScrollStrategy};
use jayjay_core::DiffHunk;
use jayjay_review::{ReviewFileRollup, ReviewFileSnapshot, ReviewGroupState, ReviewMarkSource};

use super::RepoWindow;
use crate::diff::ReviewDisplayState;
use crate::repo::view_model::{DiffLoadState, LoadedReviewSnapshot, RepoViewModel};

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

/// Mutate against fresh disk state, or saving the startup snapshot would clobber marks the CLI or SwiftUI shell persisted since load.
pub fn mutate<R>(
    store: &SharedReviewStore,
    f: impl FnOnce(&mut jayjay_review::ReviewStore) -> R,
) -> R {
    let mut store = store.borrow_mut();
    store.refresh_from_disk();
    f(&mut store)
}

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
        let snapshot = {
            let vm = self.vm.read(cx);
            hunk_with_identity(vm, &path, &identity)
                .and_then(|hunk| review_snapshot_for_hunk(vm, hunk))
        };
        mutate(&self.review_store, |store| {
            if let Some(snapshot) = snapshot.as_ref() {
                store.toggle_snapshot(&change_id, &path, &identity, snapshot);
            } else {
                store.toggle(&change_id, &path, &identity);
            }
        });
        if self.file_column.hide_reviewed {
            self.reconcile_file_selection(cx);
        }
        cx.notify();
    }

    pub fn is_reviewed(&self, change_id: &str, path: &str, identity: &str) -> bool {
        let mut store = self.review_store.borrow_mut();
        // Render-path reads must notice marks the CLI or SwiftUI shell wrote while this window was open; only mutations refresh otherwise.
        store.refresh_if_stale();
        store.is_reviewed(change_id, path, identity)
    }

    pub fn review_rollup(
        &self,
        change_id: &str,
        path: &str,
        identity: &str,
        cx: &App,
    ) -> ReviewFileRollup {
        let vm = self.vm.read(cx);
        hunk_with_identity(vm, path, identity)
            .and_then(|hunk| {
                self.review_rollups_with_vm(change_id, std::iter::once(hunk), vm)
                    .remove(path)
            })
            .unwrap_or(ReviewFileRollup::Unreviewed)
    }

    pub(crate) fn review_rollups_with_vm<'a>(
        &self,
        change_id: &str,
        hunks: impl IntoIterator<Item = &'a DiffHunk>,
        vm: &RepoViewModel,
    ) -> HashMap<String, ReviewFileRollup> {
        let hunks: Vec<_> = hunks.into_iter().collect();
        let paths: Vec<_> = hunks.iter().map(|hunk| hunk.path.clone()).collect();
        let identities: Vec<_> = hunks
            .iter()
            .map(|hunk| hunk.review_identity.clone())
            .collect();
        let snapshots: Vec<_> = hunks
            .iter()
            .map(|hunk| review_snapshot_for_hunk(vm, hunk))
            .collect();
        let mut store = self.review_store.borrow_mut();
        store.refresh_if_stale();
        let rollups = store.file_rollups(change_id, &paths, &identities, &snapshots);
        paths.into_iter().zip(rollups).collect()
    }

    pub(crate) fn agent_marked_paths(&self, change_id: &str) -> HashSet<String> {
        let mut store = self.review_store.borrow_mut();
        store.refresh_if_stale();
        store
            .paths_owned_by(change_id, ReviewMarkSource::Agent)
            .into_iter()
            .collect()
    }

    pub(crate) fn review_display_state(
        &self,
        hunk: Option<&DiffHunk>,
        file_diff: Option<&Arc<jayjay_core::diff::FileDiff>>,
        cx: &App,
    ) -> Option<Arc<ReviewDisplayState>> {
        let hunk = hunk?;
        let file_diff = file_diff?;
        let vm = self.vm.read(cx);
        if !vm.shows_review_controls()
            || hunk.projection.is_some()
            || hunk.review_identity.is_empty()
        {
            return None;
        }
        let change_id = vm.selected_change()?.change_id.id.clone();
        let review = loaded_review_snapshot(vm, hunk)?;
        let rows = self.diff.wrap_cache.borrow_mut().review_rows(file_diff);
        // The map came from the loaded text pair; if the diff has since regrouped, hide stripes instead of labelling the wrong hunks.
        if review.display_groups.len() != rows.group_count {
            return None;
        }
        let group_states = {
            let mut store = self.review_store.borrow_mut();
            store.refresh_if_stale();
            store.display_hunk_states(
                &change_id,
                &hunk.path,
                &hunk.review_identity,
                &review.snapshot,
                &review.display_groups,
            )
        };
        Some(Arc::new(ReviewDisplayState {
            path: hunk.path.clone(),
            identity: hunk.review_identity.clone(),
            group_states,
            rows,
        }))
    }

    pub fn selected_review_group_states(&self, cx: &App) -> Vec<ReviewGroupState> {
        let vm = self.vm.read(cx);
        self.review_display_state(vm.selected_hunk(), vm.shown.diff.as_ref(), cx)
            .map(|state| state.group_states.clone())
            .unwrap_or_default()
    }

    pub(crate) fn toggle_review_hunk(
        &mut self,
        expected_path: &str,
        expected_identity: &str,
        display_group: u32,
        cx: &mut Context<Self>,
    ) {
        let (change_id, path, identity, review) = {
            let vm = self.vm.read(cx);
            if !vm.shows_review_controls() {
                return;
            }
            let Some(change_id) = vm
                .selected_change()
                .map(|change| change.change_id.id.clone())
            else {
                return;
            };
            let Some(hunk) = vm.selected_hunk().filter(|hunk| {
                hunk.path == expected_path && hunk.review_identity == expected_identity
            }) else {
                return;
            };
            let Some(review) = loaded_review_snapshot(vm, hunk) else {
                return;
            };
            if review
                .display_groups
                .get(display_group as usize)
                .is_none_or(Vec::is_empty)
            {
                return;
            }
            (
                change_id,
                hunk.path.clone(),
                hunk.review_identity.clone(),
                review,
            )
        };
        mutate(&self.review_store, |store| {
            store.toggle_display_group_snapshot(
                &change_id,
                &path,
                &identity,
                &review.snapshot,
                &review.display_groups,
                display_group,
            );
        });
        cx.notify();
    }

    pub(super) fn review_file_context(&self, cx: &App) -> (bool, Option<String>) {
        let vm = self.vm.read(cx);
        let show_review = vm.shows_review_controls();
        let change_id = vm.selected_change().map(|c| c.change_id.id.clone());
        (show_review, change_id)
    }

    /// Every note surface must gate through this.
    pub fn review_notes_context(&self, hunk: &DiffHunk, cx: &App) -> Option<String> {
        let (show_review, change_id) = self.review_file_context(cx);
        if !show_review || hunk.projection.is_some() || hunk.review_identity.is_empty() {
            return None;
        }
        change_id
    }

    /// `None` in compare mode or on a non-working-copy change, which triggers clearing.
    fn review_notes_change_id(&self, cx: &App) -> Option<String> {
        let vm = self.vm.read(cx);
        vm.shows_review_controls()
            .then(|| vm.selected_change().map(|c| c.change_id.id.clone()))
            .flatten()
    }

    /// Always `refresh_if_stale()` before reading the shared store; resolved notes are included so they keep their dimmed dot.
    fn snapshot_review_notes(&self, change_id: &str) -> Vec<jayjay_review::NoteEntry> {
        let mut store = self.review_store.borrow_mut();
        store.refresh_if_stale();
        store.list_notes(change_id, true)
    }

    /// Call after mutating through `mutate()` so `vm.notes.all` and the cached row list reflect the write.
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

    /// Detects note writes this process didn't make and diff refreshes that change file identities; reconciliation depends on both.
    pub fn sync_review_notes(&mut self, cx: &mut Context<Self>) {
        let Some(change_id) = self.review_notes_change_id(cx) else {
            // Gate just turned off: drop the last key too, or `vm.notes.all` keeps serving badges for a change no longer shown.
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

    /// Identities change exactly when file bytes change, which reconciliation depends on.
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
            if !vm.shows_review_controls() {
                return;
            }
            let Some(change_id) = vm.selected_change().map(|c| c.change_id.id.clone()) else {
                return;
            };
            let Some(hunk) = vm
                .selected_hunk()
                .filter(|hunk| !hunk.review_identity.is_empty())
            else {
                return;
            };
            (
                change_id,
                hunk.path.clone(),
                hunk.review_identity.clone(),
                vm.files.clone().unwrap_or_default(),
            )
        };
        self.toggle_reviewed(change_id.clone(), path.clone(), identity, cx);
        let rollups = self.review_rollups_with_vm(&change_id, files.iter(), self.vm.read(cx));
        let reviewed = |path: &str| rollups.get(path) == Some(&ReviewFileRollup::Reviewed);
        let next_ix = reviewed(&path)
            .then(|| files.iter().position(|hunk| !reviewed(&hunk.path)))
            .flatten();
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

fn hunk_with_identity<'a>(
    vm: &'a RepoViewModel,
    path: &str,
    identity: &str,
) -> Option<&'a DiffHunk> {
    vm.files
        .as_deref()?
        .iter()
        .find(|hunk| hunk.path == path && hunk.review_identity == identity)
}

fn loaded_review_snapshot(
    vm: &RepoViewModel,
    hunk: &DiffHunk,
) -> Option<Arc<LoadedReviewSnapshot>> {
    match vm.diff_load_state(hunk) {
        DiffLoadState::Loaded(loaded) => loaded.review,
        DiffLoadState::Missing | DiffLoadState::Failed => None,
    }
}

pub(super) fn review_snapshot_for_hunk(
    vm: &RepoViewModel,
    hunk: &DiffHunk,
) -> Option<ReviewFileSnapshot> {
    loaded_review_snapshot(vm, hunk).map(|review| review.snapshot.clone())
}
