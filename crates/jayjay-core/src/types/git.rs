#[derive(Debug, Clone)]
pub struct GitSubmoduleStatus {
    pub path: String,
    pub has_new_commits: bool,
    pub has_modified_content: bool,
    pub has_untracked_content: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
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
