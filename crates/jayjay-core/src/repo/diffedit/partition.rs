use std::collections::BTreeSet;

use crate::diff::compute_file_diff_full;
use crate::types::*;

#[derive(Clone, Debug)]
struct RawLine {
    text: String,
    has_newline: bool,
}

#[derive(Debug)]
pub(super) struct PartitionedSelection {
    pub(super) selected_text: String,
    pub(super) selected_exists: bool,
    pub(super) remaining_text: String,
    pub(super) remaining_exists: bool,
    pub(super) selected_changed_lines: usize,
}

/// Split a file's diff into its selected and remaining sides for the tree-rewrite flows.
pub(super) fn partition_file_selection(
    selection: &DiffEditFileSelection,
    ignore_whitespace: bool,
) -> CoreResult<PartitionedSelection> {
    if selection.hunk_type == HunkType::Renamed || selection.old_path.is_some() {
        return Err(CoreError::Internal {
            message: format!("diff edit does not support renamed path {}", selection.path),
        });
    }

    let old_text = selection.old_content.as_deref().unwrap_or_default();
    let new_text = selection.new_content.as_deref().unwrap_or_default();
    if !crate::placeholder::is_editable_text(old_text)
        || !crate::placeholder::is_editable_text(new_text)
    {
        return Err(CoreError::Internal {
            message: format!("diff edit only supports textual files: {}", selection.path),
        });
    }

    let old_lines = split_raw_lines(old_text);
    let new_lines = split_raw_lines(new_text);
    let diff = compute_file_diff_full(&selection.path, old_text, new_text, ignore_whitespace);
    let selected_indices = selected_line_indices(&selection.line_ranges);

    let mut selected_result = Vec::new();
    let mut remaining_result = Vec::new();
    let mut selected_changed_lines = 0usize;
    let mut total_changed_lines = 0usize;

    for (index, line) in diff.lines.iter().enumerate() {
        let is_selected = selected_indices.contains(&(index + 1));
        match line.style {
            crate::diff::DiffSpanStyle::Context | crate::diff::DiffSpanStyle::Unchanged => {
                if let Some(new_line_no) = line.new_line_no {
                    let cloned = clone_line(&new_lines, new_line_no)?;
                    selected_result.push(cloned.clone());
                    remaining_result.push(cloned);
                }
            }
            crate::diff::DiffSpanStyle::Removed => {
                total_changed_lines += 1;
                if is_selected {
                    selected_changed_lines += 1;
                    if let Some(old_line_no) = line.old_line_no {
                        remaining_result.push(clone_line(&old_lines, old_line_no)?);
                    }
                } else if let Some(old_line_no) = line.old_line_no {
                    selected_result.push(clone_line(&old_lines, old_line_no)?);
                }
            }
            crate::diff::DiffSpanStyle::Added => {
                total_changed_lines += 1;
                if is_selected {
                    selected_changed_lines += 1;
                    if let Some(new_line_no) = line.new_line_no {
                        selected_result.push(clone_line(&new_lines, new_line_no)?);
                    }
                } else if let Some(new_line_no) = line.new_line_no {
                    remaining_result.push(clone_line(&new_lines, new_line_no)?);
                }
            }
            crate::diff::DiffSpanStyle::Separator => {}
        }
    }

    let selected_exists = match selection.hunk_type {
        HunkType::Added => selected_changed_lines > 0,
        HunkType::Removed => selected_changed_lines < total_changed_lines,
        HunkType::Modified => selection.old_content.is_some(),
        HunkType::Renamed => false,
    };
    let remaining_exists = match selection.hunk_type {
        HunkType::Added => selected_changed_lines < total_changed_lines,
        HunkType::Removed => selected_changed_lines > 0,
        HunkType::Modified => selection.old_content.is_some(),
        HunkType::Renamed => false,
    };

    Ok(PartitionedSelection {
        selected_text: join_raw_lines(&selected_result),
        remaining_text: join_raw_lines(&remaining_result),
        selected_exists,
        remaining_exists,
        selected_changed_lines,
    })
}

fn split_raw_lines(text: &str) -> Vec<RawLine> {
    if text.is_empty() {
        return Vec::new();
    }

    text.split_inclusive('\n')
        .map(|segment| {
            let has_newline = segment.ends_with('\n');
            let text = if has_newline {
                segment[..segment.len() - 1].to_owned()
            } else {
                segment.to_owned()
            };
            RawLine { text, has_newline }
        })
        .collect()
}

fn join_raw_lines(lines: &[RawLine]) -> String {
    let mut result = String::new();
    for line in lines {
        result.push_str(&line.text);
        if line.has_newline {
            result.push('\n');
        }
    }
    result
}

fn clone_line(lines: &[RawLine], line_no: u32) -> CoreResult<RawLine> {
    lines
        .get((line_no.saturating_sub(1)) as usize)
        .cloned()
        .ok_or_else(|| CoreError::Internal {
            message: format!("missing line {line_no} in diff selection"),
        })
}

fn selected_line_indices(ranges: &[DiffEditRange]) -> BTreeSet<usize> {
    let mut indices = BTreeSet::new();
    for range in ranges {
        let start = range.start_line.min(range.end_line) as usize;
        let end = range.start_line.max(range.end_line) as usize;
        for index in start..=end {
            indices.insert(index);
        }
    }
    indices
}

#[cfg(test)]
mod tests {
    use super::*;

    fn partition(
        hunk_type: HunkType,
        old_content: Option<&str>,
        new_content: Option<&str>,
        ranges: &[(u32, u32)],
    ) -> PartitionedSelection {
        partition_file_selection(
            &DiffEditFileSelection {
                path: "test.txt".to_owned(),
                old_path: None,
                old_content: old_content.map(str::to_owned),
                new_content: new_content.map(str::to_owned),
                hunk_type,
                line_ranges: ranges
                    .iter()
                    .map(|(start, end)| DiffEditRange {
                        start_line: *start,
                        end_line: *end,
                    })
                    .collect(),
            },
            false,
        )
        .expect("partition selection")
    }

    #[test]
    fn selecting_added_line_keeps_only_selected_change() {
        let selection = partition(
            HunkType::Modified,
            Some("a\nb\n"),
            Some("a\nx\n"),
            &[(3, 3)],
        );
        assert_eq!(selection.selected_text, "a\nb\nx\n");
        assert!(selection.selected_exists);
        assert_eq!(selection.selected_changed_lines, 1);
    }

    #[test]
    fn selecting_removed_and_added_lines_replaces_content() {
        let selection = partition(
            HunkType::Modified,
            Some("a\nb\n"),
            Some("a\nx\n"),
            &[(2, 3)],
        );
        assert_eq!(selection.selected_text, "a\nx\n");
        assert!(selection.selected_exists);
        assert_eq!(selection.selected_changed_lines, 2);
    }

    #[test]
    fn selecting_removed_line_on_deleted_file_produces_absent_selected_tree() {
        let selection = partition(HunkType::Removed, Some("a\n"), None, &[(1, 1)]);
        assert_eq!(selection.selected_text, "");
        assert!(!selection.selected_exists);
        assert_eq!(selection.remaining_text, "a\n");
        assert!(selection.remaining_exists);
    }

    #[test]
    fn selecting_part_of_added_file_keeps_partial_file() {
        let selection = partition(HunkType::Added, None, Some("a\nb\n"), &[(1, 1)]);
        assert_eq!(selection.selected_text, "a\n");
        assert!(selection.selected_exists);
        assert_eq!(selection.remaining_text, "b\n");
        assert!(selection.remaining_exists);
    }
}
