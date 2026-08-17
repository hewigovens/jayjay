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
    /// Actionable only when resolved; empty for repositories that predate recorded roots.
    pub path: String,
    pub is_path_resolved: bool,
    pub is_current: bool,
    pub change_id: ShortId,
    pub description: String,
    pub timestamp: i64,
    pub has_conflict: bool,
    pub files_changed: u32,
}

/// Not a bool: a repo that momentarily fails to load is undecided, not forgotten.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspacePresence {
    Exists,
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
