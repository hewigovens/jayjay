use jj_lib::conflicts::ConflictMaterializeOptions;
use jj_lib::files::{MergeResult, merge_hunks};
use jj_lib::merge::Merge;

use crate::diff::{FileDiff, compute_file_diff};
use crate::file_display::optional_bytes_to_display;
use crate::{CoreError, CoreResult, MergeEditorHunk, MergeHunkSource};

const HUNK_CONTEXT_LINES: usize = 3;

pub(crate) fn merge_editor_hunks<T: AsRef<[u8]>>(
    contents: &Merge<T>,
    options: &ConflictMaterializeOptions,
    materialized: &str,
) -> Vec<MergeEditorHunk> {
    if contents.num_sides() != 2 {
        return Vec::new();
    }
    let MergeResult::Conflict(hunks) = merge_hunks(contents, &options.merge) else {
        return Vec::new();
    };
    let unresolved = hunks
        .into_iter()
        .filter(|hunk| !hunk.is_resolved())
        .collect::<Vec<_>>();
    let blocks = conflict_blocks(materialized, options.marker_len.unwrap_or(7));
    if blocks.len() != unresolved.len() {
        return Vec::new();
    }

    let mut seen = std::collections::HashMap::new();
    unresolved
        .into_iter()
        .zip(blocks)
        .enumerate()
        .map(|(index, (hunk, raw))| {
            let occurrence = seen.entry(raw.to_owned()).or_insert(0u32);
            let hunk = MergeEditorHunk {
                index: index as u32,
                occurrence: *occurrence,
                raw: raw.to_owned(),
                left: optional_bytes_to_display(hunk.get_add(0)),
                base: optional_bytes_to_display(hunk.get_remove(0)),
                right: optional_bytes_to_display(hunk.get_add(1)),
            };
            *occurrence += 1;
            hunk
        })
        .collect()
}

fn raw_occurrence_starts(result: &str, raw: &str) -> Vec<usize> {
    let mut starts = Vec::new();
    if raw.is_empty() {
        return starts;
    }
    let mut from = 0;
    while let Some(found) = result[from..].find(raw) {
        starts.push(from + found);
        from += found + raw.len();
    }
    starts
}

// Clamped to the last remaining occurrence: identical blocks are interchangeable once some are resolved.
fn hunk_occurrence_start(result: &str, hunk: &MergeEditorHunk) -> Option<usize> {
    let starts = raw_occurrence_starts(result, &hunk.raw);
    let last = starts.len().checked_sub(1)?;
    Some(starts[(hunk.occurrence as usize).min(last)])
}

pub fn merge_hunk_is_unresolved(result: &str, hunk: &MergeEditorHunk) -> bool {
    hunk_occurrence_start(result, hunk).is_some()
}

pub fn merge_result_use_source(
    result: &str,
    hunk: &MergeEditorHunk,
    source: MergeHunkSource,
) -> CoreResult<String> {
    let replacement = match source {
        MergeHunkSource::Left => &hunk.left,
        MergeHunkSource::Base => &hunk.base,
        MergeHunkSource::Right => &hunk.right,
    };
    let Some(start) = hunk_occurrence_start(result, hunk) else {
        return Err(CoreError::Internal {
            message: format!(
                "conflict hunk {} changed in Raw view; switch back after restoring its markers",
                hunk.index + 1
            ),
        });
    };
    let mut updated = result.to_owned();
    updated.replace_range(start..start + hunk.raw.len(), replacement);
    Ok(updated)
}

pub fn merge_hunk_display_diff(path: &str, result: &str, hunk: &MergeEditorHunk) -> FileDiff {
    let Some(start) = hunk_occurrence_start(result, hunk) else {
        return compute_file_diff(path, &hunk.left, &hunk.right, false);
    };
    let end = start + hunk.raw.len();
    let marker_length = hunk
        .raw
        .lines()
        .next()
        .map(|line| line.bytes().take_while(|byte| *byte == b'<').count())
        .unwrap_or(7)
        .max(1);
    let before = context_before(&result[..start], marker_length);
    let after = context_after(&result[end..], marker_length);
    compute_file_diff(
        path,
        &format!("{before}{}{after}", hunk.left),
        &format!("{before}{}{after}", hunk.right),
        false,
    )
}

fn context_before(content: &str, marker_length: usize) -> String {
    let mut lines = Vec::new();
    for line in content.split_inclusive('\n').rev() {
        if is_marker_line(line, '>', marker_length) || is_marker_line(line, '<', marker_length) {
            break;
        }
        lines.push(line);
        if lines.len() == HUNK_CONTEXT_LINES {
            break;
        }
    }
    lines.into_iter().rev().collect()
}

fn context_after(content: &str, marker_length: usize) -> String {
    content
        .split_inclusive('\n')
        .take_while(|line| {
            !is_marker_line(line, '<', marker_length) && !is_marker_line(line, '>', marker_length)
        })
        .take(HUNK_CONTEXT_LINES)
        .collect()
}

fn conflict_blocks(content: &str, marker_length: usize) -> Vec<&str> {
    let marker_length = marker_length.max(1);
    let mut blocks = Vec::new();
    let mut start = None;
    let mut offset = 0;
    for line in content.split_inclusive('\n') {
        if start.is_none() && is_marker_line(line, '<', marker_length) {
            start = Some(offset);
        } else if let Some(block_start) = start
            && is_marker_line(line, '>', marker_length)
        {
            let end = offset + line.len();
            blocks.push(&content[block_start..end]);
            start = None;
        }
        offset += line.len();
    }
    blocks
}

fn is_marker_line(line: &str, marker: char, marker_length: usize) -> bool {
    let bytes = line.as_bytes();
    bytes.len() >= marker_length
        && bytes[..marker_length]
            .iter()
            .all(|byte| *byte == marker as u8)
        && bytes
            .get(marker_length)
            .is_none_or(|byte| *byte != marker as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_multiple_marker_blocks_without_consuming_resolved_text() {
        let raw =
            "before\n<<<<<<< one\na\n>>>>>>> one\nmiddle\n<<<<<<< two\nb\n>>>>>>> two\nafter\n";
        assert_eq!(
            conflict_blocks(raw, 7),
            vec![
                "<<<<<<< one\na\n>>>>>>> one\n",
                "<<<<<<< two\nb\n>>>>>>> two\n"
            ]
        );
    }

    #[test]
    fn replaces_only_the_selected_hunk() {
        let hunk = MergeEditorHunk {
            index: 0,
            occurrence: 0,
            raw: "<<<<<<< one\na\n>>>>>>> one\n".to_owned(),
            left: "left\n".to_owned(),
            base: "base\n".to_owned(),
            right: "right\n".to_owned(),
        };
        let result = format!("before\n{}after\n", hunk.raw);

        assert_eq!(
            merge_result_use_source(&result, &hunk, MergeHunkSource::Right).unwrap(),
            "before\nright\nafter\n"
        );
    }

    #[test]
    fn identical_blocks_resolve_at_their_own_occurrence() {
        let raw = "<<<<<<< a\nx\n>>>>>>> a\n";
        let hunk = |index: u32, occurrence: u32| MergeEditorHunk {
            index,
            occurrence,
            raw: raw.to_owned(),
            left: "left\n".to_owned(),
            base: "base\n".to_owned(),
            right: "right\n".to_owned(),
        };
        let first = hunk(0, 0);
        let second = hunk(1, 1);
        let result = format!("top\n{raw}middle\n{raw}bottom\n");

        assert_eq!(
            merge_result_use_source(&result, &second, MergeHunkSource::Right).unwrap(),
            format!("top\n{raw}middle\nright\nbottom\n")
        );
        assert_eq!(
            merge_result_use_source(&result, &first, MergeHunkSource::Left).unwrap(),
            format!("top\nleft\nmiddle\n{raw}bottom\n")
        );

        let after_second = merge_result_use_source(&result, &second, MergeHunkSource::Right)
            .expect("resolve second identical block first");
        assert!(merge_hunk_is_unresolved(&after_second, &first));
        assert!(merge_hunk_is_unresolved(&after_second, &second));
        assert_eq!(
            merge_result_use_source(&after_second, &first, MergeHunkSource::Left).unwrap(),
            "top\nleft\nmiddle\nright\nbottom\n"
        );

        let after_first = merge_result_use_source(&result, &first, MergeHunkSource::Left)
            .expect("resolve first identical block first");
        assert!(merge_hunk_is_unresolved(&after_first, &first));
        assert!(merge_hunk_is_unresolved(&after_first, &second));
        let resolved = merge_result_use_source(&after_first, &second, MergeHunkSource::Right)
            .expect("the remaining identical block must stay actionable");
        assert_eq!(resolved, "top\nleft\nmiddle\nright\nbottom\n");
        assert!(!merge_hunk_is_unresolved(&resolved, &first));
        assert!(!merge_hunk_is_unresolved(&resolved, &second));
    }

    #[test]
    fn display_diff_includes_nearby_unchanged_lines() {
        let raw = "<<<<<<< one\nleft\n=======\nright\n>>>>>>> two\n";
        let hunk = MergeEditorHunk {
            index: 0,
            occurrence: 0,
            raw: raw.to_owned(),
            left: "left\n".to_owned(),
            base: "base\n".to_owned(),
            right: "right\n".to_owned(),
        };
        let result = format!(
            "before 0\nbefore 1\nbefore 2\nbefore 3\n{raw}after 0\nafter 1\nafter 2\nafter 3\n"
        );

        let diff = merge_hunk_display_diff("sample.rs", &result, &hunk);
        let lines = diff
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.text.as_str())
                    .collect::<String>()
            })
            .collect::<Vec<_>>();

        assert_eq!(
            lines,
            [
                "before 1", "before 2", "before 3", "left", "right", "after 0", "after 1",
                "after 2",
            ]
        );
    }
}
