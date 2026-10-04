use jayjay_core as core;
use jayjay_core::{
    AnnotationLine, BookmarkFilterTarget, BookmarkInfo, ChecksStatus, CliStatus, FetchResult,
    FixSummary, FixToolFailure, GitSubmoduleStatus, InsertPosition, JjCommandResult, PrInfo,
    PrState, PullRequestImportPreview, PullRequestImportRemote, PullRequestImportSource,
    PullRequestImportWorkspace, RemoteBookmarkTarget, RemoteSyncStatus, RevsetCompletion,
    RevsetCompletionKind, RevsetFilter, RevsetFilterKind, RevsetFilterState, RevsetName,
    RevsetPreset, RevsetSuggestion, RevsetSuggestionKind, RevsetVocabulary, WorkspacePresence,
};

#[uniffi::remote(Record)]
pub struct FixSummary {
    pub checked_changes: u32,
    pub fixed_changes: u32,
    pub failures: Vec<FixToolFailure>,
}

#[uniffi::remote(Record)]
pub struct FixToolFailure {
    pub tool: String,
    pub path: String,
    pub message: String,
}

#[uniffi::remote(Record)]
pub struct RevsetPreset {
    pub id: String,
    pub label: String,
    pub revset: String,
}

#[uniffi::remote(Record)]
pub struct BookmarkFilterTarget {
    pub name: String,
    pub head: String,
    pub revset: String,
}

#[uniffi::remote(Record)]
pub struct RevsetFilterState {
    pub revset: String,
    pub previous: Option<String>,
    pub recent: Vec<String>,
}

#[uniffi::remote(Enum)]
pub enum RevsetCompletionKind {
    Function,
    Alias,
    Bookmark,
    Tag,
}

#[uniffi::remote(Record)]
pub struct RevsetCompletion {
    pub kind: RevsetCompletionKind,
    pub text: String,
    pub start: u32,
    pub len: u32,
}

#[uniffi::remote(Record)]
pub struct RevsetName {
    pub name: String,
    pub symbol: String,
}

#[uniffi::remote(Record)]
pub struct RevsetVocabulary {
    pub aliases: Vec<RevsetName>,
    pub bookmarks: Vec<RevsetName>,
    pub tags: Vec<RevsetName>,
}

#[uniffi::remote(Enum)]
pub enum RevsetSuggestionKind {
    Current,
    Bookmark,
    Recent,
}

#[uniffi::remote(Record)]
pub struct RevsetSuggestion {
    pub kind: RevsetSuggestionKind,
    pub title: String,
    pub revset: String,
}

#[uniffi::remote(Enum)]
pub enum RevsetFilterKind {
    Default,
    Preset,
    Bookmark,
    Custom,
}

#[uniffi::remote(Record)]
pub struct RevsetFilter {
    pub kind: RevsetFilterKind,
    pub label: String,
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
    Deleted,
}

#[uniffi::remote(Record)]
pub struct CliStatus {
    pub is_installed: bool,
    pub version: String,
    pub path: String,
}

#[uniffi::remote(Enum)]
pub enum InsertPosition {
    Before,
    After,
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
pub struct PullRequestImportPreview {
    pub pull_request: PullRequestImportSource,
    pub remote: PullRequestImportRemote,
    pub head_commit_id: String,
    pub same_repository: bool,
    pub workspace: PullRequestImportWorkspace,
    pub existing_workspace: Option<PullRequestImportWorkspace>,
}

#[uniffi::remote(Record)]
pub struct PullRequestImportSource {
    pub host: String,
    pub base_repo: String,
    pub number: u32,
    pub state: PrState,
    pub title: String,
    pub url: String,
}

#[uniffi::remote(Record)]
pub struct PullRequestImportRemote {
    pub name: String,
    pub url: String,
    pub exists: bool,
    pub bookmark: String,
}

#[uniffi::remote(Record)]
pub struct PullRequestImportWorkspace {
    pub name: String,
    pub dest: String,
}

#[uniffi::remote(Record)]
pub struct AnnotationLine {
    pub change_id: jayjay_core::ShortId,
    pub author: String,
    pub timestamp: String,
    pub line_number: u32,
    pub text: String,
}
