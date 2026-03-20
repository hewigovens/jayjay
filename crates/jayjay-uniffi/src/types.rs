use jayjay_core as core;
use jayjay_core::diff::{DiffLine, DiffSpan, DiffSpanStyle, FileDiff};
use jayjay_core::syntax::SyntaxToken;
use jayjay_core::{
    BookmarkInfo, ChangeDetail, ChangeInfo, DiffHunk, EdgeType, FileTreeEntry, GraphEdge,
    GraphEntry, HunkType, JJStatus, OpLogEntry,
};

// --- All types use uniffi::remote — no wrapper structs or From impls ---

#[uniffi::remote(Record)]
pub struct ChangeInfo {
    pub change_id: String,
    pub commit_id: String,
    pub description: String,
    pub author: String,
    pub email: String,
    pub timestamp_millis: i64,
    pub parents: Vec<String>,
    pub bookmarks: Vec<String>,
    pub is_working_copy: bool,
    pub has_conflict: bool,
    pub is_empty: bool,
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

#[uniffi::remote(Enum)]
pub enum HunkType {
    Added,
    Removed,
    Modified,
    Renamed,
}

#[uniffi::remote(Record)]
pub struct DiffHunk {
    pub path: String,
    pub old_path: Option<String>,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub hunk_type: core::HunkType,
}

#[uniffi::remote(Record)]
pub struct ChangeDetail {
    pub info: core::ChangeInfo,
    pub diff: Vec<core::DiffHunk>,
}

#[uniffi::remote(Record)]
pub struct BookmarkInfo {
    pub name: String,
    pub change_id: String,
    pub is_tracking_remote: bool,
}

#[uniffi::remote(Record)]
pub struct OpLogEntry {
    pub id: String,
    pub description: String,
    pub timestamp: String,
    pub is_current: bool,
}

#[uniffi::remote(Record)]
pub struct JJStatus {
    pub is_installed: bool,
    pub version: String,
    pub path: String,
}

#[uniffi::remote(Enum)]
pub enum DiffSpanStyle {
    Context,
    Added,
    Removed,
    Unchanged,
    Separator,
}

#[uniffi::remote(Enum)]
pub enum SyntaxToken {
    Plain,
    Keyword,
    StringLit,
    Comment,
    Number,
    Type,
    Function,
    Variable,
    Operator,
    Punctuation,
    Attribute,
}

#[uniffi::remote(Record)]
pub struct DiffSpan {
    pub text: String,
    pub style: core::diff::DiffSpanStyle,
    pub token: core::syntax::SyntaxToken,
}

#[uniffi::remote(Record)]
pub struct DiffLine {
    pub old_line_no: Option<u32>,
    pub new_line_no: Option<u32>,
    pub style: core::diff::DiffSpanStyle,
    pub spans: Vec<core::diff::DiffSpan>,
}

#[uniffi::remote(Record)]
pub struct FileDiff {
    pub path: String,
    pub language: String,
    pub lines: Vec<core::diff::DiffLine>,
}

#[uniffi::remote(Record)]
pub struct FileTreeEntry {
    pub name: String,
    pub path: String,
    pub depth: u32,
    pub hunk_index: Option<u32>,
}
