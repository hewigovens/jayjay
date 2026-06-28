use super::HunkType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffEditDestination {
    RemoveFromSource,
    MoveToWorkingCopy,
    NewChild,
    NewParallel,
}

#[derive(Debug, Clone)]
pub struct DiffEditRange {
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Debug, Clone)]
pub struct DiffEditFileSelection {
    pub path: String,
    pub old_path: Option<String>,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub hunk_type: HunkType,
    pub line_ranges: Vec<DiffEditRange>,
}
