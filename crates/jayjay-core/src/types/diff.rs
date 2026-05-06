#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub path: String,
    pub old_path: Option<String>,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub old_preview: Option<DiffPreview>,
    pub new_preview: Option<DiffPreview>,
    pub hunk_type: HunkType,
    /// Stable per-(path, content) key used by review state — computed from blob IDs.
    pub review_identity: String,
}

/// Rich-view preview for non-text diff content. Add variants as new media types land.
#[derive(Debug, Clone)]
pub enum DiffPreview {
    Image { path: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HunkType {
    Added,
    Removed,
    Modified,
    Renamed,
}

#[derive(Debug, Clone)]
pub struct DiffStats {
    pub insertions: u32,
    pub deletions: u32,
}
