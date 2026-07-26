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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiffStats {
    pub path: String,
    pub insertions: u32,
    pub deletions: u32,
}

impl FileDiffStats {
    pub fn total(&self) -> u32 {
        self.insertions + self.deletions
    }
}

pub const DIFF_EDIT_AUTO_EXPAND_MAX_FILES: usize = 30;
pub const DIFF_EDIT_AUTO_COLLAPSE_TOTAL_LINES: u64 = 1000;
pub const DIFF_EDIT_AUTO_COLLAPSE_FILE_LINES: u32 = 300;

pub fn diff_edit_starts_collapsed(file_count: usize, total_changed_lines: u64) -> bool {
    file_count > DIFF_EDIT_AUTO_EXPAND_MAX_FILES
        && total_changed_lines > DIFF_EDIT_AUTO_COLLAPSE_TOTAL_LINES
}

pub fn diff_edit_collapses_while_stats_pending(file_count: usize) -> bool {
    file_count > DIFF_EDIT_AUTO_EXPAND_MAX_FILES
}

pub fn diff_edit_auto_collapsed_paths(stats: &[FileDiffStats]) -> Vec<String> {
    if stats.len() <= DIFF_EDIT_AUTO_EXPAND_MAX_FILES {
        return Vec::new();
    }
    let total: u64 = stats.iter().map(|file| u64::from(file.total())).sum();
    stats
        .iter()
        .filter(|file| {
            total > DIFF_EDIT_AUTO_COLLAPSE_TOTAL_LINES
                || file.total() > DIFF_EDIT_AUTO_COLLAPSE_FILE_LINES
        })
        .map(|file| file.path.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(path: &str, insertions: u32, deletions: u32) -> FileDiffStats {
        FileDiffStats {
            path: path.to_owned(),
            insertions,
            deletions,
        }
    }

    fn small_files(count: usize) -> Vec<FileDiffStats> {
        (0..count).map(|i| stats(&i.to_string(), 5, 2)).collect()
    }

    #[test]
    fn diffs_at_the_file_cap_stay_expanded_even_when_huge() {
        let mut files = small_files(DIFF_EDIT_AUTO_EXPAND_MAX_FILES - 1);
        files.push(stats("huge", 5000, 5000));
        assert!(diff_edit_auto_collapsed_paths(&files).is_empty());
    }

    #[test]
    fn medium_diff_collapses_only_oversized_files() {
        let mut files = small_files(DIFF_EDIT_AUTO_EXPAND_MAX_FILES);
        files.push(stats("big", 400, 0));
        assert_eq!(diff_edit_auto_collapsed_paths(&files), ["big"]);
    }

    #[test]
    fn large_total_collapses_every_file() {
        let mut files = small_files(DIFF_EDIT_AUTO_EXPAND_MAX_FILES);
        files.push(stats("big", 900, 200));
        let collapsed = diff_edit_auto_collapsed_paths(&files);
        assert_eq!(collapsed.len(), files.len());
    }

    #[test]
    fn small_diff_with_many_files_stays_expanded() {
        assert!(
            diff_edit_auto_collapsed_paths(&small_files(DIFF_EDIT_AUTO_EXPAND_MAX_FILES + 5))
                .is_empty()
        );
    }

    #[test]
    fn entry_decision_matches_the_collapse_all_rule() {
        assert!(diff_edit_starts_collapsed(
            DIFF_EDIT_AUTO_EXPAND_MAX_FILES + 1,
            1001
        ));
        assert!(!diff_edit_starts_collapsed(
            DIFF_EDIT_AUTO_EXPAND_MAX_FILES + 1,
            1000
        ));
        assert!(!diff_edit_starts_collapsed(
            DIFF_EDIT_AUTO_EXPAND_MAX_FILES,
            50_000
        ));
        assert!(!diff_edit_starts_collapsed(0, 0));
    }

    #[test]
    fn pending_stats_collapse_only_past_the_file_cap() {
        assert!(diff_edit_collapses_while_stats_pending(
            DIFF_EDIT_AUTO_EXPAND_MAX_FILES + 1
        ));
        assert!(!diff_edit_collapses_while_stats_pending(
            DIFF_EDIT_AUTO_EXPAND_MAX_FILES
        ));
        assert!(!diff_edit_collapses_while_stats_pending(0));
    }
}
