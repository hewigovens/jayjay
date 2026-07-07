#[derive(Debug, Clone)]
pub struct DiffHunk {
    /// Repo-relative path, always `/`-separated (jj's canonical form); do not build this from `format_file_path`, which emits `\` on Windows and breaks the `/`-based file-tree and basename logic downstream.
    pub path: String,
    pub old_path: Option<String>,
    pub old: DiffContent,
    pub new: DiffContent,
    pub hunk_type: HunkType,
    /// Stable per-(path, content) key used by review state — computed from blob IDs.
    pub review_identity: String,
    pub projection: Option<DiffProjection>,
}

impl DiffHunk {
    /// A byte-identical rename: `detect_renames` cleared both sides because the content is unchanged, so there is nothing to diff and loading by the new path alone would render every line as added.
    pub fn is_content_free_rename(&self) -> bool {
        self.hunk_type == HunkType::Renamed && self.old.is_empty() && self.new.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct DiffContent {
    pub content: Option<String>,
    pub preview: Option<DiffPreview>,
}

impl DiffContent {
    pub fn new(content: Option<String>, preview: Option<DiffPreview>) -> Self {
        Self { content, preview }
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_none() && self.preview.is_none()
    }
}

#[derive(Debug, Clone)]
pub enum DiffPreview {
    Image { path: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffProjection {
    pub plugin_id: String,
    pub plugin_label: String,
    pub plugin_version: u32,
    pub mode: DiffProjectionMode,
    pub render_kind: DiffRenderKind,
    pub virtual_path: String,
    pub diagnostics: Vec<String>,
}

impl DiffProjection {
    pub fn identity_part(&self) -> String {
        format!(
            "{}:v{}:{}",
            self.plugin_id,
            self.plugin_version,
            self.mode.identity_key()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffProjectionMode {
    Raw,
    Processed,
}

impl DiffProjectionMode {
    pub fn identity_key(self) -> &'static str {
        match self {
            DiffProjectionMode::Raw => "raw",
            DiffProjectionMode::Processed => "processed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffRenderKind {
    Text,
    Markdown,
    Table,
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
