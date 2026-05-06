use jayjay_core as core;
use jayjay_core::diff::{
    CollapsedDiff, DiffLine, DiffSpan, DiffSpanStyle, DisplayLineMapping, FileDiff, SideBySideRow,
};
use jayjay_core::syntax::SyntaxToken;
use jayjay_core::{
    AnnotationLine, BookmarkInfo, ChangeDetail, ChangeInfo, ChecksStatus, CliStatus,
    DiffEditDestination, DiffEditFileSelection, DiffEditRange, DiffHunk, DiffPreview, DiffStats,
    EdgeType, EvologEntry, FetchResult, FileTreeEntry, GitSubmoduleStatus, GraphEdge, GraphEntry,
    HunkType, OpLogEntry, PrInfo, PrState, WorkspaceInfo,
};

// --- All types use uniffi::remote — no wrapper structs or From impls ---

#[uniffi::remote(Record)]
pub struct EvologEntry {
    pub change_id: String,
    pub commit_id: String,
    pub timestamp_millis: i64,
    pub operation: String,
    pub description: String,
}

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
    pub is_immutable: bool,
    pub is_divergent: bool,
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

#[uniffi::remote(Enum)]
pub enum DiffEditDestination {
    RemoveFromSource,
    MoveToWorkingCopy,
    NewChild,
    NewParallel,
}

#[uniffi::remote(Record)]
pub struct DiffEditRange {
    pub start_line: u32,
    pub end_line: u32,
}

#[uniffi::remote(Record)]
pub struct DiffEditFileSelection {
    pub path: String,
    pub old_path: Option<String>,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub hunk_type: core::HunkType,
    pub line_ranges: Vec<core::DiffEditRange>,
}

#[uniffi::remote(Record)]
pub struct DiffHunk {
    pub path: String,
    pub old_path: Option<String>,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub old_preview: Option<core::DiffPreview>,
    pub new_preview: Option<core::DiffPreview>,
    pub hunk_type: core::HunkType,
    pub review_identity: String,
}

#[uniffi::remote(Enum)]
pub enum DiffPreview {
    Image { path: String },
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
    pub description: String,
    pub is_tracking_remote: bool,
    pub is_deleted: bool,
    pub is_conflicted: bool,
    pub tracked_remotes: Vec<String>,
    pub available_remotes: Vec<String>,
}

#[uniffi::remote(Record)]
pub struct OpLogEntry {
    pub id: String,
    pub description: String,
    pub timestamp: String,
    pub is_current: bool,
}

#[uniffi::remote(Record)]
pub struct CliStatus {
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
    pub no_eof_newline: bool,
}

#[uniffi::remote(Record)]
pub struct FileDiff {
    pub path: String,
    pub language: String,
    pub lines: Vec<core::diff::DiffLine>,
    pub whitespace_only_hidden: bool,
}

#[uniffi::remote(Record)]
pub struct CollapsedDiff {
    pub diff: core::diff::FileDiff,
    pub display_to_full: Vec<core::diff::DisplayLineMapping>,
}

#[uniffi::remote(Record)]
pub struct DisplayLineMapping {
    pub display_line: u32,
    pub full_line: u32,
}

#[uniffi::remote(Record)]
pub struct SideBySideRow {
    pub old_line_no: String,
    pub old_spans: Vec<core::diff::DiffSpan>,
    pub old_style: core::diff::DiffSpanStyle,
    pub new_line_no: String,
    pub new_spans: Vec<core::diff::DiffSpan>,
    pub new_style: core::diff::DiffSpanStyle,
}

#[uniffi::remote(Record)]
pub struct WorkspaceInfo {
    pub name: String,
    pub path: String,
    pub is_current: bool,
}

#[uniffi::remote(Record)]
pub struct DiffStats {
    pub insertions: u32,
    pub deletions: u32,
}

#[uniffi::remote(Record)]
pub struct GitSubmoduleStatus {
    pub path: String,
    pub has_new_commits: bool,
    pub has_modified_content: bool,
    pub has_untracked_content: bool,
}

#[uniffi::remote(Enum)]
pub enum PrState {
    Open,
    Closed,
    Merged,
}

#[uniffi::remote(Enum)]
pub enum ChecksStatus {
    Passing,
    Failing,
    Pending,
    None,
}

#[uniffi::remote(Record)]
pub struct PrInfo {
    pub number: u32,
    pub state: core::PrState,
    pub title: String,
    pub url: String,
    pub checks: core::ChecksStatus,
}

#[uniffi::remote(Record)]
pub struct FetchResult {
    pub message: String,
    pub abandoned_bookmarks: Vec<String>,
    pub suggest_abandon_bookmarks: Vec<String>,
}

#[uniffi::remote(Record)]
pub struct AnnotationLine {
    pub change_id: String,
    pub author: String,
    pub timestamp: String,
    pub line_number: u32,
    pub text: String,
}

#[uniffi::remote(Record)]
pub struct FileTreeEntry {
    pub name: String,
    pub path: String,
    pub depth: u32,
    pub hunk_index: Option<u32>,
}
