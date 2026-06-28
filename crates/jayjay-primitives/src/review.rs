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
