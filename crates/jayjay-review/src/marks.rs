use super::store::{ReviewEntry, ReviewStore, key};

/// Snapshot of one file's marks, so shells can answer per-line lookups from memory instead of re-reading the store for every gutter line.
#[derive(Debug, Clone, Default)]
pub struct ReviewFileMarks {
    pub file_marked: bool,
    pub hunks: Vec<u32>,
}

impl ReviewStore {
    pub fn file_marks(&self, change_id: &str, path: &str, identity: &str) -> ReviewFileMarks {
        match self.state.reviewed.get(&key(change_id, path)) {
            Some(entry) if entry.identity == identity => ReviewFileMarks {
                file_marked: entry.file_marked,
                hunks: entry.hunks.clone(),
            },
            _ => ReviewFileMarks::default(),
        }
    }

    pub fn is_reviewed(&self, change_id: &str, path: &str, identity: &str) -> bool {
        let Some(entry) = self.state.reviewed.get(&key(change_id, path)) else {
            return false;
        };
        entry.file_marked && entry.identity == identity
    }

    pub fn is_hunk_reviewed(
        &self,
        change_id: &str,
        path: &str,
        identity: &str,
        hunk_idx: u32,
    ) -> bool {
        let Some(entry) = self.state.reviewed.get(&key(change_id, path)) else {
            return false;
        };
        if entry.identity != identity {
            return false;
        }
        entry.file_marked || entry.hunks.contains(&hunk_idx)
    }

    pub fn mark_reviewed(&mut self, change_id: &str, path: &str, identity: &str) {
        if identity.is_empty() {
            return;
        }
        self.state
            .reviewed
            .insert(key(change_id, path), ReviewEntry::marked_file(identity));
        self.save();
    }

    pub fn mark_unreviewed(&mut self, change_id: &str, path: &str) {
        self.state.reviewed.remove(&key(change_id, path));
        self.save();
    }

    pub fn toggle(&mut self, change_id: &str, path: &str, identity: &str) {
        if self.is_reviewed(change_id, path, identity) {
            self.mark_unreviewed(change_id, path);
        } else {
            self.mark_reviewed(change_id, path, identity);
        }
    }

    pub fn mark_hunk_reviewed(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        hunk_idx: u32,
    ) {
        if identity.is_empty() {
            return;
        }
        let k = key(change_id, path);
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
        self.save();
    }

    pub fn mark_hunk_unreviewed(&mut self, change_id: &str, path: &str, hunk_idx: u32) {
        let k = key(change_id, path);
        let Some(entry) = self.state.reviewed.get_mut(&k) else {
            return;
        };
        entry.hunks.retain(|i| *i != hunk_idx);
        // Caller calls set_reviewed_hunks if they want the surviving hunks kept.
        entry.file_marked = false;
        if entry.hunks.is_empty() {
            self.state.reviewed.remove(&k);
        }
        self.save();
    }

    pub fn toggle_hunk(&mut self, change_id: &str, path: &str, identity: &str, hunk_idx: u32) {
        if self.is_hunk_reviewed(change_id, path, identity, hunk_idx) {
            self.mark_hunk_unreviewed(change_id, path, hunk_idx);
        } else {
            self.mark_hunk_reviewed(change_id, path, identity, hunk_idx);
        }
    }

    pub fn set_reviewed_hunks(
        &mut self,
        change_id: &str,
        path: &str,
        identity: &str,
        hunk_indices: Vec<u32>,
    ) {
        if identity.is_empty() {
            return;
        }
        let k = key(change_id, path);
        if hunk_indices.is_empty() {
            self.state.reviewed.remove(&k);
        } else {
            let mut hunks = hunk_indices;
            hunks.sort_unstable();
            hunks.dedup();
            self.state
                .reviewed
                .insert(k, ReviewEntry::marked_hunks(identity, hunks));
        }
        self.save();
    }

    pub fn clear_change(&mut self, change_id: &str) {
        let prefix = format!("{change_id}|");
        self.state.reviewed.retain(|k, _| !k.starts_with(&prefix));
        self.save();
    }
}
