use std::path::PathBuf;

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone)]
pub struct ChangeDetail {
    pub info: ChangeInfo,
    pub diff: Vec<DiffHunk>,
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub path: PathBuf,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub hunk_type: HunkType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HunkType {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone)]
pub struct BookmarkInfo {
    pub name: String,
    pub change_id: String,
    pub is_tracking_remote: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("repository not found at {path}")]
    RepoNotFound { path: String },
    #[error("revision not found: {rev}")]
    RevNotFound { rev: String },
    #[error("{message}")]
    Internal { message: String },
}

pub type CoreResult<T> = Result<T, CoreError>;
