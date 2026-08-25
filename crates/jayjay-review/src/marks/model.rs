use jayjay_primitives::{ReviewFileState, ReviewGroupState};

/// Snapshot of one file's marks, so shells can answer per-line lookups from memory instead of re-reading the store for every gutter line.
#[derive(Debug, Clone, Default)]
pub struct ReviewFileMarks {
    pub file_marked: bool,
    pub hunks: Vec<u32>,
    pub group_states: Vec<ReviewGroupState>,
    pub removed_reviewed_count: u32,
}

impl ReviewFileMarks {
    pub(super) fn from_state(state: &ReviewFileState) -> Self {
        let file_marked = state.is_fully_reviewed();
        Self {
            file_marked,
            hunks: if file_marked {
                Vec::new()
            } else {
                state.reviewed_indices()
            },
            group_states: state.group_states().to_vec(),
            removed_reviewed_count: state.removed_reviewed_count,
        }
    }

    pub(super) fn is_hunk_reviewed(&self, hunk_idx: u32) -> bool {
        self.hunk_state(hunk_idx) == ReviewGroupState::Reviewed
    }

    fn hunk_state(&self, hunk_idx: u32) -> ReviewGroupState {
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
