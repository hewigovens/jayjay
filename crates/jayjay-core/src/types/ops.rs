#[derive(Debug, Clone)]
pub struct OpLogEntry {
    pub id: String,
    pub description: String,
    pub timestamp: String,
    pub is_current: bool,
}

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub name: String,
    pub path: String,
    pub is_current: bool,
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
