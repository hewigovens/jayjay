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
    pub is_immutable: bool,
    pub is_divergent: bool,
}

/// One entry in a change's evolution history (one rewrite operation).
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvologOperationKind {
    Snapshot,
    Describe,
    Rebase,
    Squash,
    Split,
    New,
    Rewrite,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvologVisibleRow {
    pub id: String,
    pub primary_index: u32,
    pub indices: Vec<u32>,
    pub entries: Vec<EvologEntry>,
    pub is_snapshot_run: bool,
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
