use jayjay_core as core;
use jayjay_core::{
    AnnotationLine, BookmarkInfo, ChecksStatus, CliStatus, FetchResult, FileTreeEntry,
    GitSubmoduleStatus, JjCommandResult, PrInfo, PrState, WorkspaceInfo,
};

#[uniffi::remote(Record)]
pub struct JjCommandResult {
    pub output: String,
    pub exit_code: i32,
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
    pub has_local_target: bool,
}

#[uniffi::remote(Record)]
pub struct CliStatus {
    pub is_installed: bool,
    pub version: String,
    pub path: String,
}

#[uniffi::remote(Record)]
pub struct WorkspaceInfo {
    pub name: String,
    pub path: String,
    pub is_current: bool,
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
    pub change_id: jayjay_core::ShortId,
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
