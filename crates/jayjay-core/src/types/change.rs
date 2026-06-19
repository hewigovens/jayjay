#[derive(Debug, Clone)]
pub struct CommitAuthor {
    pub name: String,
    pub email: String,
    pub timestamp_millis: i64,
}

impl CommitAuthor {
    pub fn new(name: impl Into<String>, email: impl Into<String>, timestamp_millis: i64) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
            timestamp_millis,
        }
    }

    pub fn empty(timestamp_millis: i64) -> Self {
        Self::new("", "", timestamp_millis)
    }
}

#[derive(Debug, Clone)]
pub struct ChangeInfo {
    pub change_id: String,
    /// Length of the shortest unique prefix of `change_id` among visible commits.
    /// Shells render that prefix highlighted and the remainder dimmed.
    pub change_id_short_len: u32,
    pub commit_id: String,
    /// Shortest unique prefix length of `commit_id` (same idea as `change_id_short_len`).
    pub commit_id_short_len: u32,
    pub description: String,
    pub author: CommitAuthor,
    pub parents: Vec<String>,
    pub bookmarks: Vec<String>,
    pub tags: Vec<String>,
    pub is_working_copy: bool,
    pub has_conflict: bool,
    pub is_empty: bool,
    pub is_immutable: bool,
    pub is_divergent: bool,
}

/// One entry in a change's evolution history (one rewrite operation).
#[derive(Debug, Clone)]
pub struct EvologEntry {
    pub change_id: String,
    pub commit_id: String,
    /// Operation timestamp (when this rewrite happened).
    pub timestamp_millis: i64,
    /// Operation summary, e.g. "snapshot working copy", "describe commit X", "rebase commit X".
    pub operation: String,
    /// Commit description at this point in evolution (often empty for snapshots).
    pub description: String,
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
    pub diff: Vec<super::DiffHunk>,
}
