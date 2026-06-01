use jayjay_core as core;
use jayjay_core::diff::{
    CollapsedDiff, ConflictBlock, ConflictBlockSection, ConflictLineKind, DiffDisplayItem,
    DiffLine, DiffSpan, DiffSpanStyle, DisplayLineMapping, FileDiff, RowSide, SideBySideRow,
    WrappedDiffLine, WrappedSbsRow, WrappedSide,
};
use jayjay_core::syntax::SyntaxToken;
use jayjay_core::{
    DiffEditDestination, DiffEditFileSelection, DiffEditRange, DiffHunk, DiffPreview, DiffStats,
    HunkType,
};

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

#[uniffi::remote(Enum)]
pub enum DiffSpanStyle {
    Context,
    Added,
    Removed,
    Unchanged,
    Separator,
}

#[uniffi::remote(Enum)]
pub enum ConflictLineKind {
    None,
    Start,
    End,
    Section,
    Content,
    Removed,
    Added,
}

#[uniffi::remote(Record)]
pub struct ConflictBlockSection {
    pub label: String,
    pub marker_line: u32,
    pub content_start: u32,
    pub line_end: u32,
    pub kind: core::diff::ConflictLineKind,
}

#[uniffi::remote(Record)]
pub struct ConflictBlock {
    pub title: String,
    pub line_start: u32,
    pub line_end: u32,
    pub sections: Vec<core::diff::ConflictBlockSection>,
}

#[uniffi::remote(Enum)]
pub enum DiffDisplayItem {
    Lines { line_start: u32, line_end: u32 },
    ConflictBlock { block: core::diff::ConflictBlock },
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
    pub conflict_kind: core::diff::ConflictLineKind,
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
pub struct RowSide {
    pub line_no: String,
    pub spans: Vec<core::diff::DiffSpan>,
    pub style: core::diff::DiffSpanStyle,
    pub conflict_kind: core::diff::ConflictLineKind,
}

#[uniffi::remote(Record)]
pub struct SideBySideRow {
    pub old: core::diff::RowSide,
    pub new: core::diff::RowSide,
    pub full_width: bool,
}

#[uniffi::remote(Record)]
pub struct WrappedDiffLine {
    pub line_ix: u32,
    pub line_len: u32,
    pub col_start: u32,
    pub col_end: u32,
    pub line: core::diff::DiffLine,
}

#[uniffi::remote(Record)]
pub struct WrappedSide {
    pub line_len: u32,
    pub col_start: u32,
    pub col_end: u32,
}

#[uniffi::remote(Record)]
pub struct WrappedSbsRow {
    pub row_ix: u32,
    pub old: core::diff::WrappedSide,
    pub new: core::diff::WrappedSide,
    pub row: core::diff::SideBySideRow,
}

#[uniffi::remote(Record)]
pub struct DiffStats {
    pub insertions: u32,
    pub deletions: u32,
}
