use jayjay_primitives::HunkType;

use crate::diff::{
    DiffSpanStyle, DisplayLineMapping, FileDiff, collapse_context_with_mapping,
    compute_file_diff_full, compute_file_diff_full_plain,
};

#[derive(Debug, Clone)]
pub struct DiffEditFile {
    pub path: String,
    pub old_path: Option<String>,
    pub hunk_type: HunkType,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    /// 1-based rows of the uncollapsed diff, which is what selections and the tree rewrite are keyed by.
    pub changed_lines: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct DiffEditFileDiff {
    pub display: FileDiff,
    pub display_to_full: Vec<DisplayLineMapping>,
    pub changed_lines: Vec<u32>,
}

impl DiffEditFileDiff {
    pub fn compute(
        path: &str,
        old_content: &str,
        new_content: &str,
        ignore_whitespace: bool,
        highlight: bool,
    ) -> Self {
        let full = if highlight {
            compute_file_diff_full(path, old_content, new_content, ignore_whitespace)
        } else {
            compute_file_diff_full_plain(path, old_content, new_content, ignore_whitespace)
        };
        let changed_lines = full
            .lines
            .iter()
            .enumerate()
            .filter(|(_, line)| matches!(line.style, DiffSpanStyle::Added | DiffSpanStyle::Removed))
            .map(|(ix, _)| ix as u32 + 1)
            .collect();
        let collapsed = collapse_context_with_mapping(&full);
        Self {
            display: collapsed.diff,
            display_to_full: collapsed.display_to_full,
            changed_lines,
        }
    }
}
