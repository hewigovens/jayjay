#[derive(Debug, Clone)]
pub struct GitSubmoduleStatus {
    pub path: String,
    pub has_new_commits: bool,
    pub has_modified_content: bool,
    pub has_untracked_content: bool,
}

#[derive(Debug, Clone)]
pub struct PrInfo {
    pub number: u32,
    pub state: String,
    pub title: String,
    pub url: String,
    pub checks_passed: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct CliStatus {
    pub is_installed: bool,
    pub version: String,
    pub path: String,
}
