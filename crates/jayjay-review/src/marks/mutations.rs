use jayjay_primitives::{ReviewFileState, ReviewGroupState};
use jj_diff::ReviewFileSnapshot;

use crate::store::{ReviewEntry, ReviewEntryState, ReviewStore, key};

impl ReviewStore {
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
        let state = ReviewFileState::fully_reviewed(fingerprints.len());
        self.write_file(change_id, path, identity, fingerprints, &state, Vec::new());
    }

    pub fn mark_unreviewed(&mut self, change_id: &str, path: &str) {
        let k = key(change_id, path);
        self.state.reviewed.remove(&k);
        self.save();
    }

    pub fn toggle(&mut self, change_id: &str, path: &str, identity: &str) {
        self.toggle_with(change_id, path, identity, None);
    }

    pub fn toggle_snapshot(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: &ReviewFileSnapshot,
    ) {
        self.toggle_with(change_id, path, identity, Some(snapshot));
    }

    fn toggle_with(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: Option<&ReviewFileSnapshot>,
    ) {
        let reviewed = self
            .current_state(change_id, path, identity, snapshot)
            .is_fully_reviewed();
        if reviewed {
            self.mark_unreviewed(change_id, path);
        } else {
            self.mark_reviewed_snapshot(change_id, path, identity, snapshot);
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
            self.mark_hunk_by_index(change_id, path, identity, hunk_idx);
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
        let Some(entry) = self.state.reviewed.get(&k) else {
            return;
        };
        let remaining = match &entry.state {
            ReviewEntryState::File => Vec::new(),
            ReviewEntryState::Hunks { indices } => indices
                .iter()
                .copied()
                .filter(|index| *index != hunk_idx)
                .collect(),
            ReviewEntryState::Groups { groups, .. } => groups
                .iter()
                .enumerate()
                .filter_map(|(index, group)| {
                    (index as u32 != hunk_idx && group.state == ReviewGroupState::Reviewed)
                        .then_some(index as u32)
                })
                .collect(),
        };
        if remaining.is_empty() {
            self.state.reviewed.remove(&k);
        } else {
            let updated =
                ReviewEntry::hunks(&entry.identity, remaining).with_extra(entry.extra.clone());
            self.state.reviewed.insert(k, updated);
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
        self.toggle_hunk_with(change_id, path, identity, None, hunk_idx);
    }

    pub fn toggle_hunk_snapshot(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: &ReviewFileSnapshot,
        hunk_idx: u32,
    ) {
        self.toggle_hunk_with(change_id, path, identity, Some(snapshot), hunk_idx);
    }

    fn toggle_hunk_with(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        snapshot: Option<&ReviewFileSnapshot>,
        hunk_idx: u32,
    ) {
        let reviewed = match snapshot {
            Some(snapshot) => {
                self.hunk_state(change_id, path, identity, snapshot, hunk_idx)
                    == ReviewGroupState::Reviewed
            }
            None => self.is_hunk_reviewed(change_id, path, identity, hunk_idx),
        };
        if reviewed {
            match snapshot {
                Some(snapshot) => self
                    .mark_hunk_unreviewed_snapshot(change_id, path, identity, snapshot, hunk_idx),
                None => self.mark_hunk_unreviewed(change_id, path, hunk_idx),
            }
        } else {
            self.mark_hunk_reviewed_snapshot(change_id, path, identity, snapshot, hunk_idx);
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
            } else {
                let entry =
                    ReviewEntry::hunks(identity, hunk_indices).with_extra(self.entry_extra(&k));
                self.state.reviewed.insert(k, entry);
            }
            self.save();
            return;
        }
        let mut state = self.aligned_state(change_id, path, identity, fingerprints);
        state.group_states_mut().fill(ReviewGroupState::Unreviewed);
        for index in hunk_indices {
            if let Some(group_state) = state.group_states_mut().get_mut(index as usize) {
                *group_state = ReviewGroupState::Reviewed;
            }
        }
        let removed = self.removed_digests(change_id, path, identity, &state, fingerprints);
        state.removed_reviewed_count = removed.len() as u32;
        self.write_file(change_id, path, identity, fingerprints, &state, removed);
    }

    pub fn clear_change(&mut self, change_id: &str) {
        let prefix = format!("{change_id}|");
        self.state.reviewed.retain(|k, _| !k.starts_with(&prefix));
        self.save();
    }
}
