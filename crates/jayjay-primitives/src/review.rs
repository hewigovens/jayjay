use serde::{Deserialize, Serialize};

use crate::{DiffHunk, HunkType, JayJayError};

pub type ReviewError = JayJayError;
pub type ReviewResult<T> = Result<T, ReviewError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteSide {
    Old,
    New,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteAnchor {
    pub change_id: String,
    pub path: String,
    pub identity: String,
    pub side: NoteSide,
    pub line: u32,
    pub anchor_excerpt: String,
    pub anchor_context: Vec<String>,
    // The whitespace mode the diff was rendered with when the note was created; reconcile must replay the same mode or the anchor can land in a different change group.
    pub ignore_whitespace: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteEntry {
    pub id: String,
    pub change_id: String,
    pub path: String,
    pub identity: String,
    pub side: NoteSide,
    pub line: u32,
    pub anchor_excerpt: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub anchor_context: Vec<String>,
    #[serde(default)]
    pub ignore_whitespace: bool,
    pub body: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    #[serde(default)]
    pub resolved: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at_ms: Option<i64>,
}

impl NoteEntry {
    pub fn new(id: String, anchor: NoteAnchor, body: &str, now_ms: i64) -> Self {
        Self {
            id,
            change_id: anchor.change_id,
            path: anchor.path,
            identity: anchor.identity,
            side: anchor.side,
            line: anchor.line,
            anchor_excerpt: anchor.anchor_excerpt,
            anchor_context: anchor.anchor_context,
            ignore_whitespace: anchor.ignore_whitespace,
            body: body.to_string(),
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
            resolved: false,
            resolved_at_ms: None,
        }
    }

    pub fn same_line(&self, anchor: &NoteAnchor) -> bool {
        self.change_id == anchor.change_id
            && self.path == anchor.path
            && self.identity == anchor.identity
            && self.side == anchor.side
            && self.line == anchor.line
    }

    pub fn update_at_anchor(&mut self, anchor: NoteAnchor, body: &str, now_ms: i64) {
        self.anchor_excerpt = anchor.anchor_excerpt;
        self.anchor_context = anchor.anchor_context;
        self.ignore_whitespace = anchor.ignore_whitespace;
        self.update_body(body, now_ms);
    }

    pub fn update_body(&mut self, body: &str, now_ms: i64) {
        self.body = body.to_string();
        self.updated_at_ms = now_ms;
    }

    pub fn resolve(&mut self, now_ms: i64) -> bool {
        if self.resolved {
            return false;
        }
        self.resolved = true;
        self.resolved_at_ms = Some(now_ms);
        self.updated_at_ms = now_ms;
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteStatus {
    Current,
    Stale,
    Orphaned,
    Resolved,
}

impl NoteStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            NoteStatus::Current => "current",
            NoteStatus::Stale => "stale",
            NoteStatus::Orphaned => "orphaned",
            NoteStatus::Resolved => "resolved",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewNoteStatus {
    pub note: NoteEntry,
    pub status: NoteStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_index: Option<u32>,
}

impl ReviewNoteStatus {
    pub fn new(note: &NoteEntry, status: NoteStatus, group_index: Option<u32>) -> Self {
        Self {
            note: note.clone(),
            status,
            group_index,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewGroupState {
    Reviewed,
    #[default]
    Unreviewed,
    ChangedSinceReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewFileRollup {
    Unreviewed,
    Partial,
    Reviewed,
    ChangedSinceReview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewGroupStates {
    /// A mark with no group snapshot to break it down: identity-only file marks, binaries, images.
    WholeFile {
        reviewed: bool,
    },
    PerGroup(Vec<ReviewGroupState>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewFileState {
    pub groups: ReviewGroupStates,
    pub removed_reviewed_count: u32,
}

impl ReviewFileState {
    pub fn from_groups(group_states: Vec<ReviewGroupState>, removed_reviewed_count: u32) -> Self {
        let groups = if group_states.is_empty() {
            ReviewGroupStates::WholeFile { reviewed: false }
        } else {
            ReviewGroupStates::PerGroup(group_states)
        };
        Self {
            groups,
            removed_reviewed_count,
        }
    }

    pub fn filled(state: ReviewGroupState, count: usize, removed_reviewed_count: u32) -> Self {
        Self::from_groups(vec![state; count], removed_reviewed_count)
    }

    pub fn fully_reviewed(count: usize) -> Self {
        if count == 0 {
            return Self::whole_file(true, 0);
        }
        Self::from_groups(vec![ReviewGroupState::Reviewed; count], 0)
    }

    pub fn whole_file(reviewed: bool, removed_reviewed_count: u32) -> Self {
        Self {
            groups: ReviewGroupStates::WholeFile { reviewed },
            removed_reviewed_count,
        }
    }

    pub fn group_states(&self) -> &[ReviewGroupState] {
        match &self.groups {
            ReviewGroupStates::PerGroup(states) => states,
            ReviewGroupStates::WholeFile { .. } => &[],
        }
    }

    pub fn group_states_mut(&mut self) -> &mut [ReviewGroupState] {
        match &mut self.groups {
            ReviewGroupStates::PerGroup(states) => states,
            ReviewGroupStates::WholeFile { .. } => &mut [],
        }
    }

    pub fn rollup(&self) -> ReviewFileRollup {
        if self.has_changed_since_review() {
            ReviewFileRollup::ChangedSinceReview
        } else if self.is_fully_reviewed() {
            ReviewFileRollup::Reviewed
        } else if self.has_partial_review() {
            ReviewFileRollup::Partial
        } else {
            ReviewFileRollup::Unreviewed
        }
    }

    pub fn is_fully_reviewed(&self) -> bool {
        self.removed_reviewed_count == 0
            && match &self.groups {
                ReviewGroupStates::WholeFile { reviewed } => *reviewed,
                ReviewGroupStates::PerGroup(states) => states
                    .iter()
                    .all(|state| *state == ReviewGroupState::Reviewed),
            }
    }

    pub fn has_changed_since_review(&self) -> bool {
        self.removed_reviewed_count > 0
            || self
                .group_states()
                .contains(&ReviewGroupState::ChangedSinceReview)
    }

    pub fn has_partial_review(&self) -> bool {
        !self.is_fully_reviewed() && self.group_states().contains(&ReviewGroupState::Reviewed)
    }

    pub fn reviewed_indices(&self) -> Vec<u32> {
        self.group_states()
            .iter()
            .enumerate()
            .filter(|(_, state)| **state == ReviewGroupState::Reviewed)
            .map(|(index, _)| index as u32)
            .collect()
    }

    pub fn state_at(&self, index: u32) -> ReviewGroupState {
        self.group_states()
            .get(index as usize)
            .copied()
            .unwrap_or(ReviewGroupState::Unreviewed)
    }
}

#[derive(Debug, Clone)]
pub struct ReviewHunk {
    pub path: String,
    pub old_path: Option<String>,
    pub hunk_type: HunkType,
    pub review_identity: String,
}

impl From<DiffHunk> for ReviewHunk {
    fn from(hunk: DiffHunk) -> Self {
        Self {
            path: hunk.path,
            old_path: hunk.old_path,
            hunk_type: hunk.hunk_type,
            review_identity: hunk.review_identity,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReviewFileDiff {
    pub old_content: Option<String>,
    pub new_content: Option<String>,
}

pub trait ReviewDiffProvider {
    fn review_hunks(&self) -> ReviewResult<Vec<ReviewHunk>>;
    fn review_file_diff(&self, hunk: &ReviewHunk) -> ReviewResult<ReviewFileDiff>;
}
