use super::change::ShortId;

#[derive(Debug, Clone)]
pub struct OpLogEntry {
    /// Operation id. Its `short_len` is the prefix unique among the listed operations (op ids have no templater `shortest()`, so it's computed in `op_log`).
    pub id: ShortId,
    pub description: String,
    pub timestamp: String,
    pub is_current: bool,
}

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub name: String,
    /// Last known checkout path. It is actionable only when `is_path_resolved`; an unresolved row keeps its name and operation so the stale registration can still be forgotten safely.
    pub path: String,
    pub is_path_resolved: bool,
    pub is_current: bool,
    /// Operation generation whose workspace name, root, and status produced this row.
    pub operation_id: String,
    /// Status of the workspace's committed `@`, read from the in-memory view without snapshotting its working copy.
    pub change_id: ShortId,
    pub description: String,
    pub timestamp: i64,
    pub has_conflict: bool,
    pub files_changed: u32,
}

/// Whether a workspace still exists. Not a bool: a repo that momentarily fails to load is undecided, not forgotten.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspacePresence {
    Exists,
    /// Proven absent: a loaded view has no working-copy commit for it, or its checkout is gone from disk.
    Gone,
    Unknown,
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
    pub change_id: ShortId,
    pub author: String,
    pub timestamp: String,
    pub line_number: u32,
    pub text: String,
}
