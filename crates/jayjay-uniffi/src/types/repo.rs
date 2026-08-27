use jayjay_core as core;
use jayjay_core::{
    AnnotationLine, BookmarkInfo, ChecksStatus, CliStatus, FetchResult, GitSubmoduleStatus,
    JjCommandResult, PrInfo, PrState, RemoteBookmarkTarget, RemoteSyncStatus, RevsetPreset,
    ShortId, WorkspaceInfo, WorkspacePresence,
};

#[uniffi::remote(Record)]
pub struct RevsetPreset {
    pub id: String,
    pub label: String,
    pub revset: String,
}

#[uniffi::remote(Record)]
pub struct JjCommandResult {
    pub output: String,
    pub exit_code: i32,
}

#[uniffi::remote(Record)]
pub struct BookmarkInfo {
    pub name: String,
    pub change_id: core::ShortId,
    pub description: String,
    pub is_tracking_remote: bool,
    pub is_deleted: bool,
    pub is_conflicted: bool,
    pub tracked_remotes: Vec<String>,
    pub available_remotes: Vec<String>,
    pub has_local_target: bool,
    pub remote_targets: Vec<RemoteBookmarkTarget>,
}

#[uniffi::remote(Record)]
pub struct RemoteBookmarkTarget {
    pub remote: String,
    pub change_id: String,
    pub description: String,
    pub status: core::RemoteSyncStatus,
    pub ahead: u32,
    pub behind: u32,
}

#[uniffi::remote(Enum)]
pub enum RemoteSyncStatus {
    Synced,
    Ahead,
    Behind,
    Diverged,
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
    pub is_path_resolved: bool,
    pub is_current: bool,
    pub change_id: ShortId,
    pub description: String,
    pub timestamp: i64,
    pub has_conflict: bool,
    pub files_changed: u32,
}

#[uniffi::remote(Enum)]
pub enum WorkspacePresence {
    Exists,
    Gone,
    Unknown,
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
