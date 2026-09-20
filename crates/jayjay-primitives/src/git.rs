#[derive(Debug, Clone)]
pub struct GitSubmoduleStatus {
    pub path: String,
    pub has_new_commits: bool,
    pub has_modified_content: bool,
    pub has_untracked_content: bool,
}

/// Domain PR state. Each host module owns its own wire enum (GitHubPrState, CodebergPrState) and converts into this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrState {
    Open,
    Closed,
    Merged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksStatus {
    Passing,
    Failing,
    Pending,
    None,
}

#[derive(Debug, Clone)]
pub struct PrInfo {
    pub number: u32,
    pub state: PrState,
    pub title: String,
    pub url: String,
    pub checks: ChecksStatus,
}

#[derive(Debug, Clone)]
pub struct PullRequestImportPreview {
    pub pull_request: PullRequestImportSource,
    pub remote: PullRequestImportRemote,
    pub head_commit_id: String,
    pub same_repository: bool,
    pub workspace: PullRequestImportWorkspace,
    pub existing_workspace: Option<PullRequestImportWorkspace>,
}

#[derive(Debug, Clone)]
pub struct PullRequestImportSource {
    pub host: String,
    pub base_repo: String,
    pub number: u32,
    pub state: PrState,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct PullRequestImportRemote {
    pub name: String,
    pub url: String,
    pub exists: bool,
    pub bookmark: String,
}

#[derive(Debug, Clone)]
pub struct PullRequestImportWorkspace {
    pub name: String,
    pub dest: String,
}

#[derive(Debug, Clone)]
pub struct FetchResult {
    pub message: String,
    pub abandoned_bookmarks: Vec<String>,
    pub suggest_abandon_bookmarks: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CliStatus {
    pub is_installed: bool,
    pub version: String,
    pub path: String,
}
