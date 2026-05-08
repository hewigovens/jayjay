use crate::diff::{DiffSpanStyle, FileDiff, is_editable_text, is_git_lfs, is_git_submodule};
use crate::{DiffEditFileSelection, DiffEditRange, DiffHunk, HunkType};

pub fn diff_edit_changed_lines(diff: &FileDiff) -> Vec<u32> {
    diff.lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| match line.style {
            DiffSpanStyle::Added | DiffSpanStyle::Removed => Some((index + 1) as u32),
            _ => None,
        })
        .collect()
}

pub fn diff_edit_supports_file(
    hunk_type: HunkType,
    old_content: Option<&str>,
    new_content: Option<&str>,
) -> bool {
    hunk_type != HunkType::Renamed && is_editable_text(old_content) && is_editable_text(new_content)
}

pub fn diff_edit_unsupported_reason(
    hunk_type: HunkType,
    old_content: Option<&str>,
    new_content: Option<&str>,
) -> Option<String> {
    if hunk_type == HunkType::Renamed {
        return Some("renamed files cannot be split by line yet".to_owned());
    }
    if is_git_lfs(old_content) || is_git_lfs(new_content) {
        return Some("Git LFS pointer placeholder".to_owned());
    }
    if is_git_submodule(old_content) || is_git_submodule(new_content) {
        return Some("submodule pointer change".to_owned());
    }
    if !is_editable_text(old_content) || !is_editable_text(new_content) {
        return Some("binary, conflicted, directory, or inaccessible content".to_owned());
    }
    None
}

pub fn build_diff_edit_file_selection(
    hunk: &DiffHunk,
    diff: &FileDiff,
    old_content: Option<String>,
    new_content: Option<String>,
    selected_lines: &[u32],
    inverse: bool,
) -> Option<DiffEditFileSelection> {
    if !diff_edit_supports_file(
        hunk.hunk_type,
        old_content.as_deref(),
        new_content.as_deref(),
    ) {
        return None;
    }

    let changed_lines = diff_edit_changed_lines(diff);
    let selected: std::collections::BTreeSet<u32> = selected_lines.iter().copied().collect();
    let line_numbers = changed_lines
        .into_iter()
        .filter(|line| selected.contains(line) != inverse)
        .collect::<Vec<_>>();
    let line_ranges = collapse_ranges(&line_numbers);
    if line_ranges.is_empty() {
        return None;
    }

    Some(DiffEditFileSelection {
        path: hunk.path.clone(),
        old_path: hunk.old_path.clone(),
        old_content,
        new_content,
        hunk_type: hunk.hunk_type,
        line_ranges,
    })
}

fn collapse_ranges(line_numbers: &[u32]) -> Vec<DiffEditRange> {
    let Some((&first, rest)) = line_numbers.split_first() else {
        return Vec::new();
    };
    let mut ranges = Vec::new();
    let mut start = first;
    let mut previous = first;

    for &line_number in rest {
        if line_number == previous + 1 {
            previous = line_number;
            continue;
        }
        ranges.push(DiffEditRange {
            start_line: start,
            end_line: previous,
        });
        start = line_number;
        previous = line_number;
    }
    ranges.push(DiffEditRange {
        start_line: start,
        end_line: previous,
    });
    ranges
}

#[cfg(test)]
mod tests {
    use crate::diff::{DiffLine, DiffSpanStyle};

    use super::*;

    #[test]
    fn changed_lines_are_one_based_added_or_removed() {
        let diff = FileDiff {
            path: "file.txt".to_owned(),
            language: "text".to_owned(),
            lines: vec![
                line(DiffSpanStyle::Context),
                line(DiffSpanStyle::Removed),
                line(DiffSpanStyle::Added),
            ],
            whitespace_only_hidden: false,
        };
        assert_eq!(diff_edit_changed_lines(&diff), vec![2, 3]);
    }

    #[test]
    fn builds_selected_and_inverse_ranges() {
        let hunk = DiffHunk {
            path: "file.txt".to_owned(),
            old_path: None,
            old_content: Some("a\nb\n".to_owned()),
            new_content: Some("a\nc\n".to_owned()),
            old_preview: None,
            new_preview: None,
            hunk_type: HunkType::Modified,
            review_identity: "identity".to_owned(),
        };
        let diff = FileDiff {
            path: "file.txt".to_owned(),
            language: "text".to_owned(),
            lines: vec![
                line(DiffSpanStyle::Removed),
                line(DiffSpanStyle::Added),
                line(DiffSpanStyle::Context),
                line(DiffSpanStyle::Added),
            ],
            whitespace_only_hidden: false,
        };

        let selected = build_diff_edit_file_selection(
            &hunk,
            &diff,
            hunk.old_content.clone(),
            hunk.new_content.clone(),
            &[1, 2],
            false,
        )
        .unwrap();
        assert_eq!(selected.line_ranges[0].start_line, 1);
        assert_eq!(selected.line_ranges[0].end_line, 2);

        let inverse = build_diff_edit_file_selection(
            &hunk,
            &diff,
            hunk.old_content.clone(),
            hunk.new_content.clone(),
            &[1, 2],
            true,
        )
        .unwrap();
        assert_eq!(inverse.line_ranges[0].start_line, 4);
    }

    #[test]
    fn reports_placeholder_reasons() {
        assert_eq!(
            diff_edit_unsupported_reason(HunkType::Modified, Some("<git lfs pointer>"), None),
            Some("Git LFS pointer placeholder".to_owned())
        );
        assert_eq!(
            diff_edit_unsupported_reason(HunkType::Modified, Some("<git submodule abc>"), None),
            Some("submodule pointer change".to_owned())
        );
        assert_eq!(
            diff_edit_unsupported_reason(HunkType::Modified, Some("<binary file 12 bytes>"), None),
            Some("binary, conflicted, directory, or inaccessible content".to_owned())
        );
    }

    fn line(style: DiffSpanStyle) -> DiffLine {
        DiffLine {
            old_line_no: None,
            new_line_no: None,
            style,
            spans: Vec::new(),
            no_eof_newline: false,
        }
    }
}
