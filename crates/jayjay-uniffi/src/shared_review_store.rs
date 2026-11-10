use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use jayjay_core::diff::ReviewFileSnapshot;
use jayjay_core::{JayError, ReviewPrune};
use jayjay_primitives::{
    NoteAnchor, NoteEntry, ReviewFileRollup, ReviewGroupState, ReviewMarkSource, ReviewStoreSummary,
};
use jayjay_review::{ReviewFileMarks, ReviewStore};

/// One loaded copy of the shared review store per window: reads re-parse the file only after another writer changed it, and mutations still reload under the store lock.
#[derive(uniffi::Object)]
pub struct SharedReviewStore {
    store: Mutex<ReviewStore>,
}

impl SharedReviewStore {
    fn store(&self) -> MutexGuard<'_, ReviewStore> {
        self.store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn fresh(&self) -> MutexGuard<'_, ReviewStore> {
        let mut store = self.store();
        store.refresh_if_stale();
        store
    }
}

#[uniffi::export]
impl SharedReviewStore {
    #[uniffi::constructor]
    fn new(store_path: Option<String>) -> Arc<Self> {
        let store = match store_path.filter(|path| !path.is_empty()) {
            Some(path) => ReviewStore::load_from(PathBuf::from(path)),
            None => ReviewStore::load(),
        };
        Arc::new(Self {
            store: Mutex::new(store),
        })
    }

    fn file_marks(&self, change_id: String, path: String, identity: String) -> ReviewFileMarks {
        self.fresh().file_marks(&change_id, &path, &identity, None)
    }

    fn file_rollups(
        &self,
        change_id: String,
        paths: Vec<String>,
        identities: Vec<String>,
        snapshots: Vec<Option<ReviewFileSnapshot>>,
    ) -> Vec<ReviewFileRollup> {
        self.fresh()
            .file_rollups(&change_id, &paths, &identities, &snapshots)
    }

    fn agent_marked_paths(&self, change_id: String) -> Vec<String> {
        self.fresh()
            .paths_owned_by(&change_id, ReviewMarkSource::Agent)
    }

    fn display_hunk_states(
        &self,
        change_id: String,
        path: String,
        identity: String,
        snapshot: ReviewFileSnapshot,
        mapping: Vec<Vec<u32>>,
    ) -> Vec<ReviewGroupState> {
        self.fresh()
            .display_hunk_states(&change_id, &path, &identity, &snapshot, &mapping)
    }

    fn summary(&self) -> ReviewStoreSummary {
        self.fresh().summary()
    }

    fn list_notes(&self, change_id: String, include_resolved: bool) -> Vec<NoteEntry> {
        self.fresh().list_notes(&change_id, include_resolved)
    }

    fn mark_reviewed(
        &self,
        change_id: String,
        path: String,
        identity: String,
        snapshot: Option<ReviewFileSnapshot>,
    ) {
        self.store()
            .mark_reviewed_snapshot(&change_id, &path, &identity, snapshot.as_ref());
    }

    fn mark_unreviewed(&self, change_id: String, path: String) {
        self.store().mark_unreviewed(&change_id, &path);
    }

    fn toggle_display_hunk_snapshot(
        &self,
        change_id: String,
        path: String,
        identity: String,
        snapshot: ReviewFileSnapshot,
        mapping: Vec<Vec<u32>>,
        display_index: u32,
    ) {
        self.store().toggle_display_group_snapshot(
            &change_id,
            &path,
            &identity,
            &snapshot,
            &mapping,
            display_index,
        );
    }

    fn toggle_hunk(
        &self,
        change_id: String,
        path: String,
        identity: String,
        hunk_index: u32,
        snapshot: Option<ReviewFileSnapshot>,
    ) {
        let mut store = self.store();
        match snapshot.as_ref() {
            Some(snapshot) => {
                store.toggle_hunk_snapshot(&change_id, &path, &identity, snapshot, hunk_index)
            }
            None => store.toggle_hunk(&change_id, &path, &identity, hunk_index),
        }
    }

    fn set_reviewed_hunks(
        &self,
        change_id: String,
        path: String,
        identity: String,
        hunk_indices: Vec<u32>,
        snapshot: Option<ReviewFileSnapshot>,
    ) {
        self.store().set_reviewed_hunks_snapshot(
            &change_id,
            &path,
            &identity,
            snapshot.as_ref(),
            hunk_indices,
        );
    }

    /// Opens every repository before touching the store, so a slow repository never holds the store lock.
    fn prune_preview(&self, repo_paths: Vec<String>) -> Result<ReviewStoreSummary, JayError> {
        let prune = ReviewPrune::for_repositories(&repo_paths)?;
        Ok(prune.preview(&self.fresh()))
    }

    fn prune(&self, repo_paths: Vec<String>) -> Result<ReviewStoreSummary, JayError> {
        let prune = ReviewPrune::for_repositories(&repo_paths)?;
        prune.apply(&mut self.store())
    }

    fn clear_all(&self) {
        self.store().clear_all();
    }

    fn clear_change(&self, change_id: String) {
        self.store().clear_change(&change_id);
    }

    fn add_note(&self, anchor: NoteAnchor, body: String) -> NoteEntry {
        self.store().add_note(anchor, &body)
    }

    fn update_note(&self, id: String, body: String) -> Option<NoteEntry> {
        self.store().update_note(&id, &body)
    }

    fn delete_note(&self, id: String) -> bool {
        self.store().delete_note(&id)
    }

    fn resolve_note(&self, id: String) -> Option<NoteEntry> {
        self.store().resolve_note(&id)
    }
}
