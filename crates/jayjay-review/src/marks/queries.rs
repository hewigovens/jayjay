use jayjay_primitives::{ReviewFileRollup, ReviewFileState, ReviewGroupState};
use jj_diff::{ReviewFileSnapshot, ReviewGroupFingerprint};

use super::ReviewFileMarks;
use crate::file_state::{display_group_states, reconcile};
use crate::store::{ReviewEntryState, ReviewStore, key};

impl ReviewStore {
    pub(crate) fn file_state(
        &self,
        change_id: &str,
        path: &str,
        identity: &str,
        current: Option<&[ReviewGroupFingerprint]>,
    ) -> ReviewFileState {
        let k = key(change_id, path);
        reconcile(self.state.reviewed.get(&k), identity, current)
    }

    pub(super) fn current_state(
        &self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: Option<&ReviewFileSnapshot>,
    ) -> ReviewFileState {
        self.file_state(
            change_id,
            path,
            identity,
            snapshot.map(|snapshot| snapshot.fingerprints.as_slice()),
        )
    }

    pub fn file_marks(
        &self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: Option<&ReviewFileSnapshot>,
    ) -> ReviewFileMarks {
        let mut marks =
            ReviewFileMarks::from_state(&self.current_state(change_id, path, identity, snapshot));
        if snapshot.is_none()
            && marks.group_states.is_empty()
            && let Some(entry) = self.state.reviewed.get(&key(change_id, path))
            && entry.identity == identity
        {
            match &entry.state {
                ReviewEntryState::File => marks.file_marked = true,
                ReviewEntryState::Hunks { indices } if marks.hunks.is_empty() => {
                    marks.hunks = indices.clone();
                }
                ReviewEntryState::Hunks { .. } | ReviewEntryState::Groups { .. } => {}
            }
        }
        marks
    }

    pub fn is_reviewed(&self, change_id: &str, path: &str, identity: &str) -> bool {
        self.current_state(change_id, path, identity, None)
            .is_fully_reviewed()
    }

    pub(crate) fn file_rollup(
        &self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: Option<&ReviewFileSnapshot>,
    ) -> ReviewFileRollup {
        self.current_state(change_id, path, identity, snapshot)
            .rollup()
    }

    pub fn file_rollups(
        &self,
        change_id: &str,
        paths: &[String],
        identities: &[String],
        snapshots: &[Option<ReviewFileSnapshot>],
    ) -> Vec<ReviewFileRollup> {
        paths
            .iter()
            .zip(identities)
            .enumerate()
            .map(|(index, (path, identity))| {
                self.file_rollup(
                    change_id,
                    path,
                    identity,
                    snapshots.get(index).and_then(|snapshot| snapshot.as_ref()),
                )
            })
            .collect()
    }

    pub fn display_hunk_states(
        &self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: &ReviewFileSnapshot,
        mapping: &[Vec<u32>],
    ) -> Vec<ReviewGroupState> {
        display_group_states(
            &self.current_state(change_id, path, identity, Some(snapshot)),
            mapping,
        )
    }

    pub fn is_hunk_reviewed(
        &self,
        change_id: &str,
        path: &str,
        identity: &str,
        hunk_idx: u32,
    ) -> bool {
        self.file_marks(change_id, path, identity, None)
            .is_hunk_reviewed(hunk_idx)
    }

    pub(super) fn hunk_state(
        &self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: &ReviewFileSnapshot,
        hunk_idx: u32,
    ) -> ReviewGroupState {
        self.current_state(change_id, path, identity, Some(snapshot))
            .state_at(hunk_idx)
    }
}
