#[derive(Debug, Clone)]
pub struct DiffHunk {
    /// Repo-relative path, always `/`-separated (jj's canonical form); do not build this from `format_file_path`, which emits `\` on Windows and breaks the `/`-based file-tree and basename logic downstream.
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

impl DiffHunk {
    /// A byte-identical rename: `detect_renames` cleared both sides because the content is unchanged, so there is nothing to diff and loading by the new path alone would render every line as added.
    pub fn is_content_free_rename(&self) -> bool {
        self.hunk_type == HunkType::Renamed
            && self.old_content.is_none()
            && self.new_content.is_none()
            && self.old_preview.is_none()
            && self.new_preview.is_none()
    }
}

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
    pub files_changed: u32,
    pub insertions: u32,
    pub deletions: u32,
}
