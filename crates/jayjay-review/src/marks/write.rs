use jayjay_primitives::{ReviewFileState, ReviewGroupState, ReviewGroupStates};
use jj_diff::ReviewGroupFingerprint;

use crate::file_state::{persisted_entry, unmatched_reviewed_digests};
use crate::store::{ReviewEntry, ReviewEntryState, ReviewStore, key};

impl ReviewStore {
    pub(super) fn set_hunk_state(
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

    pub(super) fn set_hunk_states(
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
        let mut state = self.aligned_state(change_id, path, identity, fingerprints);
        for hunk_idx in hunk_indices {
            if let Some(group_state) = state.group_states_mut().get_mut(*hunk_idx as usize) {
                *group_state = new_state;
            }
        }
        let removed = self.removed_digests(change_id, path, identity, &state, fingerprints);
        state.removed_reviewed_count = removed.len() as u32;
        if state
            .group_states()
            .iter()
            .all(|group| *group == ReviewGroupState::Unreviewed)
            && removed.is_empty()
        {
            self.mark_unreviewed(change_id, path);
            return;
        }
        self.write_file(change_id, path, identity, fingerprints, &state, removed);
    }

    pub(super) fn aligned_state(
        &self,
        change_id: &str,
        path: &str,
        identity: &str,
        fingerprints: &[ReviewGroupFingerprint],
    ) -> ReviewFileState {
        let mut state = self.file_state(change_id, path, identity, Some(fingerprints));
        if state.group_states().len() != fingerprints.len() {
            state.groups =
                ReviewGroupStates::PerGroup(vec![ReviewGroupState::Unreviewed; fingerprints.len()]);
        }
        state
    }

    /// Tombstones for reviewed groups the current snapshot no longer contains. Reviewing every current group clears them: the user has reviewed the whole diff, so nothing is left for the warning to point at.
    pub(super) fn removed_digests(
        &self,
        change_id: &str,
        path: &str,
        identity: &str,
        state: &ReviewFileState,
        fingerprints: &[ReviewGroupFingerprint],
    ) -> Vec<String> {
        if !state.group_states().is_empty()
            && state
                .group_states()
                .iter()
                .all(|group| *group == ReviewGroupState::Reviewed)
        {
            return Vec::new();
        }
        let k = key(change_id, path);
        unmatched_reviewed_digests(self.state.reviewed.get(&k), identity, fingerprints)
    }

    pub(super) fn entry_extra(
        &self,
        entry_key: &str,
    ) -> serde_json::Map<String, serde_json::Value> {
        self.state
            .reviewed
            .get(entry_key)
            .map(|entry| entry.extra.clone())
            .unwrap_or_default()
    }

    pub(super) fn mark_hunk_by_index(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        hunk_idx: u32,
    ) {
        let k = key(change_id, path);
        match self.state.reviewed.get_mut(&k) {
            Some(entry) if entry.identity == identity => match &mut entry.state {
                ReviewEntryState::File => return,
                ReviewEntryState::Hunks { indices } => {
                    if !indices.contains(&hunk_idx) {
                        indices.push(hunk_idx);
                        indices.sort_unstable();
                    }
                }
                ReviewEntryState::Groups { groups, .. } => {
                    let mut indices: Vec<u32> = groups
                        .iter()
                        .enumerate()
                        .filter_map(|(index, group)| {
                            (group.state == ReviewGroupState::Reviewed).then_some(index as u32)
                        })
                        .collect();
                    if !indices.contains(&hunk_idx) {
                        indices.push(hunk_idx);
                        indices.sort_unstable();
                    }
                    entry.state = ReviewEntryState::Hunks { indices };
                }
            },
            _ => {
                let entry =
                    ReviewEntry::hunks(identity, vec![hunk_idx]).with_extra(self.entry_extra(&k));
                self.state.reviewed.insert(k, entry);
            }
        }
        self.save();
    }

    pub(super) fn write_file(
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
            && state.group_states().is_empty()
            && !state.is_fully_reviewed()
            && removed_reviewed.is_empty();
        if empty {
            self.state.reviewed.remove(&k);
            self.save();
            return;
        }
        let entry = persisted_entry(identity, fingerprints, state, removed_reviewed)
            .with_extra(self.entry_extra(&k));
        self.state.reviewed.insert(k, entry);
        self.save();
    }
}
