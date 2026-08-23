use jayjay_primitives::{ReviewFileRollup, ReviewFileState, ReviewGroupState};
use jj_diff::{ReviewFileSnapshot, ReviewGroupFingerprint};

use super::file_state::{
    copy_group_extras, display_group_states, persist_reconciled, reconcile,
    unmatched_reviewed_digests,
};
use super::store::{ReviewEntry, ReviewStore, key};

/// Snapshot of one file's marks, so shells can answer per-line lookups from memory instead of re-reading the store for every gutter line.
#[derive(Debug, Clone, Default)]
pub struct ReviewFileMarks {
    pub file_marked: bool,
    pub hunks: Vec<u32>,
    pub group_states: Vec<ReviewGroupState>,
    pub removed_reviewed_count: u32,
}

impl ReviewFileMarks {
    pub fn from_state(state: &ReviewFileState) -> Self {
        let file_marked = state.is_fully_reviewed();
        Self {
            file_marked,
            hunks: if file_marked {
                Vec::new()
            } else {
                state.reviewed_indices()
            },
            group_states: state.group_states.clone(),
            removed_reviewed_count: state.removed_reviewed_count,
        }
    }

    pub fn is_hunk_reviewed(&self, hunk_idx: u32) -> bool {
        self.hunk_state(hunk_idx) == ReviewGroupState::Reviewed
    }

    pub fn hunk_state(&self, hunk_idx: u32) -> ReviewGroupState {
        if let Some(state) = self.group_states.get(hunk_idx as usize) {
            return *state;
        }
        if self.file_marked || self.hunks.contains(&hunk_idx) {
            ReviewGroupState::Reviewed
        } else {
            ReviewGroupState::Unreviewed
        }
    }
}

impl ReviewStore {
    pub fn file_state(
        &self,
        change_id: &str,
        path: &str,
        identity: &str,
        current: Option<&[ReviewGroupFingerprint]>,
    ) -> ReviewFileState {
        let k = key(change_id, path);
        reconcile(
            self.state.reviewed.get(&k),
            self.state.review_baselines.get(&k),
            identity,
            current,
        )
    }

    pub fn file_marks(&self, change_id: &str, path: &str, identity: &str) -> ReviewFileMarks {
        let mut marks = ReviewFileMarks::from_state(&self.file_state(change_id, path, identity, None));
        if marks.group_states.is_empty()
            && let Some(entry) = self.state.reviewed.get(&key(change_id, path))
            && entry.identity == identity
        {
            marks.file_marked = entry.file_marked || marks.file_marked;
            if marks.hunks.is_empty() {
                marks.hunks = entry.hunks.clone();
            }
        }
        marks
    }

    pub fn file_marks_with_snapshot(
        &self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: &ReviewFileSnapshot,
    ) -> ReviewFileMarks {
        ReviewFileMarks::from_state(&self.file_state(
            change_id,
            path,
            identity,
            Some(snapshot.fingerprints.as_slice()),
        ))
    }

    pub fn is_reviewed(&self, change_id: &str, path: &str, identity: &str) -> bool {
        self.file_state(change_id, path, identity, None)
            .is_fully_reviewed()
    }

    pub fn file_rollup(&self, change_id: &str, path: &str, identity: &str) -> ReviewFileRollup {
        self.file_state(change_id, path, identity, None).rollup()
    }

    pub fn file_rollup_with_snapshot(
        &self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: &ReviewFileSnapshot,
    ) -> ReviewFileRollup {
        self.file_state(
            change_id,
            path,
            identity,
            Some(snapshot.fingerprints.as_slice()),
        )
        .rollup()
    }

    pub fn file_rollups(
        &self,
        change_id: &str,
        paths: &[String],
        identities: &[String],
    ) -> Vec<ReviewFileRollup> {
        paths
            .iter()
            .zip(identities)
            .map(|(path, identity)| self.file_rollup(change_id, path, identity))
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
            &self.file_state(
                change_id,
                path,
                identity,
                Some(snapshot.fingerprints.as_slice()),
            ),
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
        self.file_marks(change_id, path, identity)
            .is_hunk_reviewed(hunk_idx)
    }

    pub fn hunk_state(
        &self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: &ReviewFileSnapshot,
        hunk_idx: u32,
    ) -> ReviewGroupState {
        self.file_state(
            change_id,
            path,
            identity,
            Some(snapshot.fingerprints.as_slice()),
        )
        .state_at(hunk_idx)
    }

    pub fn mark_reviewed(&mut self, change_id: &str, path: &str, identity: &str) {
        self.mark_reviewed_snapshot(change_id, path, identity, None);
    }

    pub fn mark_reviewed_snapshot(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: Option<&ReviewFileSnapshot>,
    ) {
        if identity.is_empty() {
            return;
        }
        let fingerprints = snapshot.map(|s| s.fingerprints.as_slice()).unwrap_or(&[]);
        let state = ReviewFileState {
            group_states: fingerprints
                .iter()
                .map(|_| ReviewGroupState::Reviewed)
                .collect(),
            removed_reviewed_count: 0,
            file_marked: true,
        };
        self.write_file(change_id, path, identity, fingerprints, &state, Vec::new());
    }

    pub fn mark_unreviewed(&mut self, change_id: &str, path: &str) {
        let k = key(change_id, path);
        self.state.reviewed.remove(&k);
        self.state.review_baselines.remove(&k);
        self.save();
    }

    pub fn toggle(&mut self, change_id: &str, path: &str, identity: &str) {
        if self.is_reviewed(change_id, path, identity) {
            self.mark_unreviewed(change_id, path);
        } else {
            self.mark_reviewed(change_id, path, identity);
        }
    }

    pub fn toggle_snapshot(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: &ReviewFileSnapshot,
    ) {
        if self
            .file_state(
                change_id,
                path,
                identity,
                Some(snapshot.fingerprints.as_slice()),
            )
            .is_fully_reviewed()
        {
            self.mark_unreviewed(change_id, path);
        } else {
            self.mark_reviewed_snapshot(change_id, path, identity, Some(snapshot));
        }
    }

    pub fn mark_hunk_reviewed(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        hunk_idx: u32,
    ) {
        self.mark_hunk_reviewed_snapshot(change_id, path, identity, None, hunk_idx);
    }

    pub fn mark_hunk_reviewed_snapshot(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: Option<&ReviewFileSnapshot>,
        hunk_idx: u32,
    ) {
        if identity.is_empty() {
            return;
        }
        let fingerprints = snapshot.map(|s| s.fingerprints.as_slice()).unwrap_or(&[]);
        if fingerprints.is_empty() {
            self.mark_hunk_legacy(change_id, path, identity, hunk_idx, true);
            return;
        }
        self.set_hunk_state(
            change_id,
            path,
            identity,
            fingerprints,
            hunk_idx,
            ReviewGroupState::Reviewed,
        );
    }

    pub fn mark_hunk_unreviewed(&mut self, change_id: &str, path: &str, hunk_idx: u32) {
        let k = key(change_id, path);
        let Some(entry) = self.state.reviewed.get_mut(&k) else {
            return;
        };
        entry.hunks.retain(|i| *i != hunk_idx);
        entry.file_marked = false;
        if entry.hunks.is_empty() {
            self.state.reviewed.remove(&k);
            self.state.review_baselines.remove(&k);
        }
        self.save();
    }

    pub fn mark_hunk_unreviewed_snapshot(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: &ReviewFileSnapshot,
        hunk_idx: u32,
    ) {
        self.set_hunk_state(
            change_id,
            path,
            identity,
            snapshot.fingerprints.as_slice(),
            hunk_idx,
            ReviewGroupState::Unreviewed,
        );
    }

    pub fn toggle_hunk(&mut self, change_id: &str, path: &str, identity: &str, hunk_idx: u32) {
        if self.is_hunk_reviewed(change_id, path, identity, hunk_idx) {
            self.mark_hunk_unreviewed(change_id, path, hunk_idx);
        } else {
            self.mark_hunk_reviewed(change_id, path, identity, hunk_idx);
        }
    }

    pub fn toggle_hunk_snapshot(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: &ReviewFileSnapshot,
        hunk_idx: u32,
    ) {
        if self.hunk_state(change_id, path, identity, snapshot, hunk_idx)
            == ReviewGroupState::Reviewed
        {
            self.mark_hunk_unreviewed_snapshot(change_id, path, identity, snapshot, hunk_idx);
        } else {
            self.mark_hunk_reviewed_snapshot(
                change_id,
                path,
                identity,
                Some(snapshot),
                hunk_idx,
            );
        }
    }

    pub fn toggle_display_group_snapshot(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: &ReviewFileSnapshot,
        mapping: &[Vec<u32>],
        display_index: u32,
    ) {
        let Some(indices) = mapping.get(display_index as usize) else {
            return;
        };
        if indices.is_empty() {
            return;
        }
        let display_states = self.display_hunk_states(change_id, path, identity, snapshot, mapping);
        let reviewed =
            display_states.get(display_index as usize) == Some(&ReviewGroupState::Reviewed);
        let new_state = if reviewed {
            ReviewGroupState::Unreviewed
        } else {
            ReviewGroupState::Reviewed
        };
        self.set_hunk_states(
            change_id,
            path,
            identity,
            snapshot.fingerprints.as_slice(),
            indices,
            new_state,
        );
    }

    pub fn set_reviewed_hunks(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        hunk_indices: Vec<u32>,
    ) {
        self.set_reviewed_hunks_snapshot(change_id, path, identity, None, hunk_indices);
    }

    pub fn set_reviewed_hunks_snapshot(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: Option<&ReviewFileSnapshot>,
        hunk_indices: Vec<u32>,
    ) {
        if identity.is_empty() {
            return;
        }
        let fingerprints = snapshot.map(|s| s.fingerprints.as_slice()).unwrap_or(&[]);
        if fingerprints.is_empty() {
            let k = key(change_id, path);
            if hunk_indices.is_empty() {
                self.state.reviewed.remove(&k);
                self.state.review_baselines.remove(&k);
            } else {
                let mut hunks = hunk_indices;
                hunks.sort_unstable();
                hunks.dedup();
                self.state
                    .reviewed
                    .insert(k, ReviewEntry::marked_hunks(identity, hunks));
                self.state.review_baselines.remove(&key(change_id, path));
            }
            self.save();
            return;
        }
        let mut state = self.file_state(change_id, path, identity, Some(fingerprints));
        if state.group_states.len() != fingerprints.len() {
            state.group_states = vec![ReviewGroupState::Unreviewed; fingerprints.len()];
        }
        for group_state in &mut state.group_states {
            *group_state = ReviewGroupState::Unreviewed;
        }
        for index in hunk_indices {
            if let Some(group_state) = state.group_states.get_mut(index as usize) {
                *group_state = ReviewGroupState::Reviewed;
            }
        }
        state.removed_reviewed_count = 0;
        state.file_marked = state.is_fully_reviewed();
        let removed = if state.file_marked {
            Vec::new()
        } else {
            self.removed_digests(change_id, path, fingerprints, &state)
        };
        self.write_file(change_id, path, identity, fingerprints, &state, removed);
    }

    pub fn clear_change(&mut self, change_id: &str) {
        let prefix = format!("{change_id}|");
        self.state.reviewed.retain(|k, _| !k.starts_with(&prefix));
        self.state
            .review_baselines
            .retain(|k, _| !k.starts_with(&prefix));
        self.save();
    }

    fn set_hunk_state(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        fingerprints: &[ReviewGroupFingerprint],
        hunk_idx: u32,
        new_state: ReviewGroupState,
    ) {
        self.set_hunk_states(
            change_id,
            path,
            identity,
            fingerprints,
            &[hunk_idx],
            new_state,
        );
    }

    fn set_hunk_states(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        fingerprints: &[ReviewGroupFingerprint],
        hunk_indices: &[u32],
        new_state: ReviewGroupState,
    ) {
        if hunk_indices.is_empty() {
            return;
        }
        let mut state = self.file_state(change_id, path, identity, Some(fingerprints));
        if state.group_states.len() != fingerprints.len() {
            state.group_states = vec![ReviewGroupState::Unreviewed; fingerprints.len()];
        }
        for hunk_idx in hunk_indices {
            if let Some(group_state) = state.group_states.get_mut(*hunk_idx as usize) {
                *group_state = new_state;
            }
        }
        let removed = self.removed_digests(change_id, path, fingerprints, &state);
        state.removed_reviewed_count = removed.len() as u32;
        state.file_marked = state.is_fully_reviewed();
        if !state.file_marked
            && state
                .group_states
                .iter()
                .all(|group| *group == ReviewGroupState::Unreviewed)
            && removed.is_empty()
        {
            self.mark_unreviewed(change_id, path);
            return;
        }
        self.write_file(change_id, path, identity, fingerprints, &state, removed);
    }

    fn removed_digests(
        &self,
        change_id: &str,
        path: &str,
        fingerprints: &[ReviewGroupFingerprint],
        state: &ReviewFileState,
    ) -> Vec<String> {
        let k = key(change_id, path);
        unmatched_reviewed_digests(
            self.state.reviewed.get(&k),
            self.state.review_baselines.get(&k),
            fingerprints,
            &state.group_states,
        )
    }

    fn mark_hunk_legacy(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        hunk_idx: u32,
        reviewed: bool,
    ) {
        let k = key(change_id, path);
        if reviewed {
            match self.state.reviewed.get_mut(&k) {
                Some(entry) if entry.identity == identity => {
                    if !entry.hunks.contains(&hunk_idx) {
                        entry.hunks.push(hunk_idx);
                        entry.hunks.sort_unstable();
                    }
                }
                _ => {
                    self.state
                        .reviewed
                        .insert(k, ReviewEntry::marked_hunks(identity, vec![hunk_idx]));
                }
            }
        } else {
            self.mark_hunk_unreviewed(change_id, path, hunk_idx);
            return;
        }
        self.save();
    }

    fn write_file(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        fingerprints: &[ReviewGroupFingerprint],
        state: &ReviewFileState,
        removed_reviewed: Vec<String>,
    ) {
        if identity.is_empty() {
            return;
        }
        let k = key(change_id, path);
        let empty = fingerprints.is_empty()
            && state.group_states.is_empty()
            && !state.file_marked
            && removed_reviewed.is_empty();
        if empty {
            self.state.reviewed.remove(&k);
            self.state.review_baselines.remove(&k);
            self.save();
            return;
        }
        let (mut entry, mut baseline) =
            persist_reconciled(identity, fingerprints, state, removed_reviewed);
        if let Some(existing) = self.state.reviewed.get(&k) {
            entry.extra = existing.extra.clone();
        }
        if let Some(existing) = self.state.review_baselines.get(&k) {
            baseline.extra = existing.extra.clone();
            copy_group_extras(&existing.groups, &mut baseline.groups);
        }
        if fingerprints.is_empty() {
            self.state.reviewed.insert(k.clone(), entry);
            if state.file_marked {
                self.state.review_baselines.insert(k, baseline);
            } else {
                self.state.review_baselines.remove(&k);
            }
        } else {
            self.state.reviewed.insert(k.clone(), entry);
            self.state.review_baselines.insert(k, baseline);
        }
        self.save();
    }
}
