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
    pub has_conflict: bool,
    pub is_empty: bool,
}

/// A change with its graph edges for DAG rendering.
#[derive(Debug, Clone)]
pub struct GraphEntry {
    pub change: ChangeInfo,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    /// Target commit_id (hex) this edge points to.
    pub target: String,
    pub edge_type: EdgeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    Direct,
    Indirect,
    Missing,
}

#[derive(Debug, Clone)]
pub struct ChangeDetail {
    pub info: ChangeInfo,
    pub diff: Vec<DiffHunk>,
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub path: String,
    pub old_path: Option<String>,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub hunk_type: HunkType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HunkType {
    Added,
    Removed,
    Modified,
    Renamed,
}

#[derive(Debug, Clone)]
pub struct BookmarkInfo {
    pub name: String,
    pub change_id: String,
    pub is_tracking_remote: bool,
    pub tracked_remotes: Vec<String>,
    pub available_remotes: Vec<String>,
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

#[derive(Debug, Clone)]
pub struct OpLogEntry {
    pub id: String,
    pub description: String,
    pub timestamp: String,
    pub is_current: bool,
}

#[derive(Debug, Clone)]
pub struct JJStatus {
    pub is_installed: bool,
    pub version: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct FileTreeEntry {
    pub name: String,
    pub path: String,
    pub depth: u32,
    /// If Some, this is a file entry with associated hunk index. If None, it's a directory.
    pub hunk_index: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct AnnotationLine {
    pub change_id: String,
    pub author: String,
    pub timestamp: String,
    pub line_number: u32,
    pub text: String,
}

pub type CoreResult<T> = Result<T, CoreError>;
