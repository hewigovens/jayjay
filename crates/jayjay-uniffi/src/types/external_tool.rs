use jayjay_core as core;
use jayjay_core::external_tools::{
    ExternalDiffFile, ExternalDiffSelection, ExternalDiffSide, ExternalMerge,
    ExternalToolInvocation,
};

#[uniffi::remote(Enum)]
pub enum ExternalToolInvocation {
    Diff {
        left: String,
        right: String,
        editable: bool,
    },
    Merge {
        left: String,
        base: String,
        right: String,
        output: String,
        path: String,
        marker_length: u32,
        output_is_initialized: bool,
    },
}

#[uniffi::remote(Record)]
pub struct ExternalDiffFile {
    pub hunk: core::DiffHunk,
    pub topology_group: Option<String>,
    pub display_diff: core::diff::FileDiff,
    pub display_to_full: Vec<core::diff::DisplayLineMapping>,
    pub changed_lines: Vec<u32>,
    pub supports_editing: bool,
    pub old_exists: bool,
    pub new_exists: bool,
    pub old_executable: Option<bool>,
    pub new_executable: Option<bool>,
    pub stats: core::FileDiffStats,
}

#[uniffi::remote(Record)]
pub struct ExternalDiffSelection {
    pub file: core::DiffEditFileSelection,
    pub selected_exists: bool,
    pub selected_executable: Option<bool>,
    pub whole_file_side: Option<ExternalDiffSide>,
}

#[uniffi::remote(Enum)]
pub enum ExternalDiffSide {
    Old,
    New,
}

#[uniffi::remote(Record)]
pub struct ExternalMerge {
    pub left: String,
    pub base: String,
    pub right: String,
    pub result: String,
    pub is_text: bool,
    pub hunks: Vec<core::MergeEditorHunk>,
}
