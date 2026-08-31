use jayjay_core as core;
use jayjay_core::{
    ChangeDetail, ChangeInfo, CommitAuthor, EdgeType, EvologEntry, EvologRow, GraphEdge,
    GraphEntry, OpLogEntry, ShortId,
};

#[uniffi::remote(Record)]
pub struct ShortId {
    pub id: String,
    pub short_len: u32,
}

#[uniffi::remote(Record)]
pub struct EvologEntry {
    pub change_id: core::ShortId,
    pub commit_id: core::ShortId,
    pub timestamp_millis: i64,
    pub operation: String,
    pub description: String,
}

#[uniffi::remote(Record)]
pub struct EvologRow {
    pub start: u32,
    pub count: u32,
}

#[uniffi::remote(Record)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
    pub timestamp_millis: i64,
}

#[uniffi::remote(Record)]
pub struct ChangeInfo {
    pub change_id: core::ShortId,
    pub commit_id: core::ShortId,
    pub description: String,
    pub author: core::CommitAuthor,
    pub parents: Vec<String>,
    pub bookmarks: Vec<String>,
    pub tags: Vec<String>,
    pub workspaces: Vec<String>,
    pub is_working_copy: bool,
    pub has_conflict: bool,
    pub is_empty: bool,
    pub is_immutable: bool,
    pub is_divergent: bool,
    pub new_change: core::NewChangeEligibility,
}

use jayjay_core::NewChangeEligibility;

#[uniffi::remote(Record)]
pub struct NewChangeEligibility {
    pub on_top: bool,
    pub before: bool,
    pub after: bool,
}

#[uniffi::remote(Record)]
pub struct GraphEntry {
    pub change: core::ChangeInfo,
    pub edges: Vec<core::GraphEdge>,
}

#[uniffi::remote(Record)]
pub struct GraphEdge {
    pub target: String,
    pub edge_type: core::EdgeType,
}

#[uniffi::remote(Enum)]
pub enum EdgeType {
    Direct,
    Indirect,
    Missing,
}

#[uniffi::remote(Record)]
pub struct ChangeDetail {
    pub info: core::ChangeInfo,
    pub diff: Vec<core::DiffHunk>,
}

#[uniffi::remote(Record)]
pub struct OpLogEntry {
    pub id: core::ShortId,
    pub description: String,
    pub timestamp: String,
    pub is_current: bool,
}
