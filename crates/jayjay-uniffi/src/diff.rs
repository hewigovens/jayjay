#[cfg(feature = "desktop")]
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use jayjay_core::FileDiffStats;
use jayjay_core::diff::{
    self, ChangeGroup, CollapsedDiff, ConflictLineKind, ContextExpansion, ContextExpansionResult,
    DiffLine, DiffSpan, FileDiff, SideBySideRow, WrappedDiffLine, WrappedSbsRow,
};
#[cfg(feature = "desktop")]
use jayjay_core::{
    DiffEditRange, MergeEditorHunk, MergeHunkSource,
    external_tools::{
        ExternalDiffFile, ExternalDiffSelection, ExternalMerge, ExternalToolInvocation,
    },
};

use jayjay_core::diff::ContextExpansionError;

#[cfg(feature = "desktop")]
use crate::error::JayJayError;

#[cfg(feature = "desktop")]
#[uniffi::export]
fn parse_external_tool_invocation(
    arguments: Vec<String>,
) -> Result<Option<ExternalToolInvocation>, JayJayError> {
    jayjay_core::external_tools::parse_external_tool_invocation(&arguments)
        .map_err(JayJayError::from)
}

#[cfg(feature = "desktop")]
#[uniffi::export]
fn external_tool_cancel_exit_code(invocation: ExternalToolInvocation) -> i32 {
    invocation.cancel_exit_code()
}

#[cfg(feature = "desktop")]
#[uniffi::export]
fn load_external_diff(
    left: String,
    right: String,
    editable: bool,
) -> Result<Vec<ExternalDiffFile>, JayJayError> {
    jayjay_core::external_tools::load_external_diff(
        &PathBuf::from(left),
        &PathBuf::from(right),
        editable,
    )
    .map_err(JayJayError::from)
}

#[cfg(feature = "desktop")]
#[uniffi::export]
fn load_external_merge(
    left: String,
    base: String,
    right: String,
    output: String,
    marker_length: u32,
) -> Result<ExternalMerge, JayJayError> {
    jayjay_core::external_tools::load_external_merge(
        &PathBuf::from(left),
        &PathBuf::from(base),
        &PathBuf::from(right),
        &PathBuf::from(output),
        marker_length as usize,
    )
    .map_err(JayJayError::from)
}

#[cfg(feature = "desktop")]
#[uniffi::export]
fn diff_edit_ranges(lines: Vec<u32>) -> Vec<DiffEditRange> {
    jayjay_core::external_tools::diff_edit_ranges(lines)
}

#[cfg(feature = "desktop")]
#[uniffi::export]
fn apply_external_diff(
    left: String,
    right: String,
    selections: Vec<ExternalDiffSelection>,
    ignore_whitespace: bool,
) -> Result<(), JayJayError> {
    jayjay_core::external_tools::apply_external_diff_selections(
        &PathBuf::from(left),
        &PathBuf::from(right),
        &selections,
        ignore_whitespace,
    )
    .map_err(JayJayError::from)
}

#[cfg(feature = "desktop")]
#[uniffi::export]
fn write_external_merge(output: String, content: String) -> Result<(), JayJayError> {
    jayjay_core::external_tools::save_external_merge(
        &PathBuf::from(output),
        jayjay_core::external_tools::ExternalMergeResolution::Content(&content),
    )
    .map_err(JayJayError::from)
}

#[cfg(feature = "desktop")]
#[uniffi::export]
fn use_external_merge_side(source: String, output: String) -> Result<(), JayJayError> {
    jayjay_core::external_tools::save_external_merge(
        &PathBuf::from(output),
        jayjay_core::external_tools::ExternalMergeResolution::Source(&PathBuf::from(source)),
    )
    .map_err(JayJayError::from)
}

#[cfg(feature = "desktop")]
#[uniffi::export]
fn external_conflict_marker_count(content: String, marker_length: u32) -> u64 {
    jayjay_core::external_tools::conflict_marker_count(&content, marker_length as usize) as u64
}

#[cfg(feature = "desktop")]
#[uniffi::export]
fn merge_result_use_source(
    result: String,
    hunk: MergeEditorHunk,
    source: MergeHunkSource,
) -> Result<String, JayJayError> {
    jayjay_core::merge_result_use_source(&result, &hunk, source).map_err(JayJayError::from)
}

#[cfg(feature = "desktop")]
#[uniffi::export]
fn merge_hunk_is_unresolved(result: String, hunk: MergeEditorHunk) -> bool {
    jayjay_core::merge_hunk_is_unresolved(&result, &hunk)
}

#[derive(uniffi::Object)]
pub struct ExpandableDiff {
    inner: Mutex<diff::ExpandableDiff>,
}

#[uniffi::export]
fn make_expandable_diff(
    diff: FileDiff,
    old_content: String,
    new_content: String,
) -> Arc<ExpandableDiff> {
    Arc::new(ExpandableDiff {
        inner: Mutex::new(diff::ExpandableDiff::new(diff, old_content, new_content)),
    })
}

#[uniffi::export]
impl ExpandableDiff {
    fn expand(
        &self,
        region_id: u32,
        expansion: ContextExpansion,
    ) -> Result<ContextExpansionResult, ContextExpansionError> {
        self.inner
            .lock()
            .map_err(|_| ContextExpansionError::SessionUnavailable)?
            .expand(region_id, expansion)
    }

    fn expand_all(&self) -> Result<ContextExpansionResult, ContextExpansionError> {
        self.inner
            .lock()
            .map_err(|_| ContextExpansionError::SessionUnavailable)?
            .expand_all()
    }
}

#[uniffi::export]
fn diff_edit_auto_collapsed_paths(stats: Vec<FileDiffStats>) -> Vec<String> {
    jayjay_core::diff_edit_auto_collapsed_paths(&stats)
}

#[uniffi::export]
fn diff_edit_starts_collapsed(file_count: u64, total_changed_lines: u64) -> bool {
    jayjay_core::diff_edit_starts_collapsed(file_count as usize, total_changed_lines)
}

#[uniffi::export]
fn diff_edit_collapses_while_stats_pending(file_count: u64) -> bool {
    jayjay_core::diff_edit_collapses_while_stats_pending(file_count as usize)
}

#[uniffi::export]
fn build_side_by_side_rows(lines: Vec<DiffLine>) -> Vec<SideBySideRow> {
    diff::build_side_by_side_rows(&lines)
}

#[uniffi::export]
fn wrap_cols_for_width(width: f32, advance: f32) -> u32 {
    diff::wrap_cols_for_width(width, advance)
}

#[uniffi::export]
fn wrap_diff_lines(lines: Vec<DiffLine>, cols: u32) -> Vec<WrappedDiffLine> {
    diff::wrap_diff_lines(&lines, cols)
}

#[uniffi::export]
fn wrap_sbs_rows(rows: Vec<SideBySideRow>, old_cols: u32, new_cols: u32) -> Vec<WrappedSbsRow> {
    diff::wrap_sbs_rows(&rows, old_cols, new_cols)
}

#[uniffi::export]
fn conflict_display_text(kind: ConflictLineKind, raw: String) -> Option<String> {
    diff::conflict_display_text(kind, &raw)
}

#[uniffi::export]
fn diff_display_lines(lines: Vec<DiffLine>) -> Vec<DiffLine> {
    diff::build_diff_display_lines(&lines)
}

#[uniffi::export]
fn compute_file_diff(
    path: String,
    old_content: String,
    new_content: String,
    ignore_whitespace: bool,
) -> FileDiff {
    diff::compute_file_diff(&path, &old_content, &new_content, ignore_whitespace)
}

#[uniffi::export]
fn compute_file_diff_full(
    path: String,
    old_content: String,
    new_content: String,
    ignore_whitespace: bool,
) -> FileDiff {
    diff::compute_file_diff_full(&path, &old_content, &new_content, ignore_whitespace)
}

#[uniffi::export]
fn compute_file_diff_full_plain(
    path: String,
    old_content: String,
    new_content: String,
    ignore_whitespace: bool,
) -> FileDiff {
    diff::compute_file_diff_full_plain(&path, &old_content, &new_content, ignore_whitespace)
}

#[uniffi::export]
fn collapse_diff_with_mapping(diff: FileDiff) -> CollapsedDiff {
    diff::collapse_context_with_mapping(&diff)
}

#[uniffi::export]
fn change_groups(lines: Vec<DiffLine>) -> Vec<ChangeGroup> {
    diff::change_groups(&lines)
}

#[uniffi::export]
fn highlight_file_lines(path: String, content: String) -> Vec<Vec<DiffSpan>> {
    diff::highlight_file(&path, &content)
}

#[uniffi::export]
fn highlight_file_against_base(path: String, base: String, content: String) -> Vec<DiffLine> {
    diff::highlight_file_against_base(&path, &base, &content)
}

#[cfg(feature = "desktop")]
#[uniffi::export]
fn merge_hunk_display_diff(path: String, result: String, hunk: MergeEditorHunk) -> FileDiff {
    jayjay_core::merge_hunk_display_diff(&path, &result, &hunk)
}

// `visual_index_for_*` stay Rust-only — exporting would copy the full wrapped Vec across FFI per lookup.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expandable_diff_object_reveals_context_repeatedly() {
        let old_lines: Vec<String> = (1..=80).map(|line| format!("line {line}")).collect();
        let mut new_lines = old_lines.clone();
        new_lines[39] = "changed".to_owned();
        let old = old_lines.join("\n") + "\n";
        let new = new_lines.join("\n") + "\n";
        let diff = diff::compute_file_diff("sample.txt", &old, &new, false);
        let region = diff
            .lines
            .iter()
            .find_map(|line| line.context_region)
            .unwrap();
        let expandable = make_expandable_diff(diff, old, new);

        let first = expandable
            .expand(region.id, ContextExpansion::ShowMore { line_count: 10 })
            .unwrap();
        let second = expandable
            .expand(region.id, ContextExpansion::ShowMore { line_count: 10 })
            .unwrap();

        assert_eq!(first.inserted.count, 10);
        assert_eq!(second.inserted.count, 10);
        assert_eq!(second.diff.lines.len(), first.diff.lines.len() + 10);
    }

    #[test]
    fn expandable_diff_object_reports_stale_region() {
        let old = (1..=30)
            .map(|line| format!("line {line}"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        let mut new = old.clone();
        new = new.replace("line 15", "changed");
        let diff = diff::compute_file_diff("sample.txt", &old, &new, false);
        let region = diff
            .lines
            .iter()
            .find_map(|line| line.context_region)
            .unwrap();
        let expandable = make_expandable_diff(diff, old, new);

        expandable
            .expand(region.id, ContextExpansion::ShowAll)
            .unwrap();
        assert!(matches!(
            expandable.expand(region.id, ContextExpansion::ShowAll),
            Err(ContextExpansionError::UnknownRegion { region_id }) if region_id == region.id
        ));
    }

    #[test]
    fn standalone_diff_does_not_require_a_repository() {
        let diff = compute_file_diff(
            "src/lib.rs".into(),
            "fn old() {}\n".into(),
            "fn new() {}\n".into(),
            false,
        );

        assert!(
            diff.lines
                .iter()
                .any(|line| line.old_line_no.is_some() && line.new_line_no.is_none())
        );
        assert!(
            diff.lines
                .iter()
                .any(|line| line.old_line_no.is_none() && line.new_line_no.is_some())
        );
    }
}
