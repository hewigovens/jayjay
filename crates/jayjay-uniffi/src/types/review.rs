use jayjay_core::diff::{ReviewFileSnapshot, ReviewGroupFingerprint};
use jayjay_primitives as primitives;
use jayjay_primitives::{
    NoteAnchor, NoteEntry, NoteSide, NoteStatus, ReviewFileRollup, ReviewGroupState,
    ReviewNoteStatus,
};
use jayjay_review::ReviewFileMarks;

#[uniffi::remote(Enum)]
pub enum NoteSide {
    Old,
    New,
}

#[uniffi::remote(Record)]
pub struct NoteAnchor {
    pub change_id: String,
    pub path: String,
    pub identity: String,
    pub side: primitives::NoteSide,
    pub line: u32,
    pub anchor_excerpt: String,
    pub anchor_context: Vec<String>,
    pub ignore_whitespace: bool,
}

#[uniffi::remote(Record)]
pub struct NoteEntry {
    pub id: String,
    pub change_id: String,
    pub path: String,
    pub identity: String,
    pub side: primitives::NoteSide,
    pub line: u32,
    pub anchor_excerpt: String,
    pub anchor_context: Vec<String>,
    pub ignore_whitespace: bool,
    pub body: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub resolved: bool,
    pub resolved_at_ms: Option<i64>,
}

#[uniffi::remote(Enum)]
pub enum NoteStatus {
    Current,
    Stale,
    Orphaned,
    Resolved,
}

#[uniffi::remote(Record)]
pub struct ReviewNoteStatus {
    pub note: primitives::NoteEntry,
    pub status: primitives::NoteStatus,
    pub group_index: Option<u32>,
}

#[uniffi::remote(Enum)]
pub enum ReviewGroupState {
    Reviewed,
    Unreviewed,
    ChangedSinceReview,
}

#[uniffi::remote(Enum)]
pub enum ReviewFileRollup {
    Unreviewed,
    Partial,
    Reviewed,
    ChangedSinceReview,
}

#[uniffi::remote(Record)]
pub struct ReviewGroupFingerprint {
    pub digest: String,
}

#[uniffi::remote(Record)]
pub struct ReviewFileSnapshot {
    pub algorithm_version: u32,
    pub fingerprints: Vec<jayjay_core::diff::ReviewGroupFingerprint>,
}

#[uniffi::remote(Record)]
pub struct ReviewFileMarks {
    pub file_marked: bool,
    pub hunks: Vec<u32>,
    pub group_states: Vec<primitives::ReviewGroupState>,
    pub removed_reviewed_count: u32,
}
