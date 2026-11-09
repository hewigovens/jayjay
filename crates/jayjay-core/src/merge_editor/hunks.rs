use jj_lib::conflicts::ConflictMaterializeOptions;
use jj_lib::files::{MergeResult, merge_hunks};
use jj_lib::merge::Merge;

use crate::diff::{FileDiff, compute_file_diff};
use crate::file_display::optional_bytes_to_display;
use crate::{JayError, JayResult, MergeEditorHunk, MergeHunkPick, MergeHunkSource};

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

/// Applies `pick` to `result`. While `result` is exactly the loaded markers with `picks` applied, it is rebuilt from the original markers, so a resolved hunk can switch sides.
pub fn pick_merge_hunk(
    original: &str,
    hunks: &[MergeEditorHunk],
    picks: &[MergeHunkPick],
    result: &str,
    pick: MergeHunkPick,
) -> JayResult<String> {
    let hunk = hunks
        .iter()
        .find(|hunk| hunk.index == pick.index)
        .ok_or_else(|| JayError::internal(format!("no conflict hunk {}", pick.index + 1)))?;
    if result_with_picks(original, hunks, picks) != result {
        return hunk.use_source(result, pick.source);
    }
    let mut picks = picks.to_vec();
    picks.retain(|existing| existing.index != pick.index);
    picks.push(pick);
    Ok(result_with_picks(original, hunks, &picks))
}

fn result_with_picks(original: &str, hunks: &[MergeEditorHunk], picks: &[MergeHunkPick]) -> String {
    let mut replacements = picks
        .iter()
        .filter_map(|pick| {
            let hunk = hunks.iter().find(|hunk| hunk.index == pick.index)?;
            let start = hunk.occurrence_start(original)?;
            Some((start..start + hunk.raw.len(), side(hunk, pick.source)))
        })
        .collect::<Vec<_>>();
    replacements.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    let mut result = original.to_owned();
    for (range, side) in replacements {
        result.replace_range(range, side);
    }
    result
}

fn side(hunk: &MergeEditorHunk, source: MergeHunkSource) -> &str {
    match source {
        MergeHunkSource::Left => &hunk.left,
        MergeHunkSource::Base => &hunk.base,
        MergeHunkSource::Right => &hunk.right,
    }
}

pub trait MergeEditorHunkExt {
    fn use_source(&self, result: &str, source: MergeHunkSource) -> JayResult<String>;
    fn display_diff(&self, path: &str, result: &str) -> FileDiff;
}

impl MergeEditorHunkExt for MergeEditorHunk {
    fn use_source(&self, result: &str, source: MergeHunkSource) -> JayResult<String> {
        let replacement = side(self, source);
        let Some(start) = self.occurrence_start(result) else {
            return Err(JayError::Internal {
                message: format!(
                    "conflict hunk {} changed in Raw view; switch back after restoring its markers",
                    self.index + 1
                ),
            });
        };
        let mut updated = result.to_owned();
        updated.replace_range(start..start + self.raw.len(), replacement);
        Ok(updated)
    }

    fn display_diff(&self, path: &str, result: &str) -> FileDiff {
        let Some(start) = self.occurrence_start(result) else {
            return compute_file_diff(path, &self.left, &self.right, false);
        };
        let end = start + self.raw.len();
        let marker_length = self
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
            &format!("{before}{}{after}", self.left),
            &format!("{before}{}{after}", self.right),
            false,
        )
    }
}

fn context_before(content: &str, marker_length: usize) -> String {
    let mut lines = Vec::new();
    for line in content.split_inclusive('\n').rev() {
        if is_conflict_marker_line(line, b'>', marker_length)
            || is_conflict_marker_line(line, b'<', marker_length)
        {
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
            !is_conflict_marker_line(line, b'<', marker_length)
                && !is_conflict_marker_line(line, b'>', marker_length)
        })
        .take(HUNK_CONTEXT_LINES)
        .collect()
}

fn conflict_blocks(content: &str, marker_length: usize) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut start = None;
    let mut offset = 0;
    for line in content.split_inclusive('\n') {
        if start.is_none() && is_conflict_marker_line(line, b'<', marker_length) {
            start = Some(offset);
        } else if let Some(block_start) = start
            && is_conflict_marker_line(line, b'>', marker_length)
        {
            let end = offset + line.len();
            blocks.push(&content[block_start..end]);
            start = None;
        }
        offset += line.len();
    }
    blocks
}

pub(crate) fn is_conflict_marker_line(line: &str, marker: u8, marker_length: usize) -> bool {
    let marker_length = marker_length.max(1);
    let bytes = line.as_bytes();
    bytes.len() >= marker_length
        && bytes[..marker_length].iter().all(|byte| *byte == marker)
        && bytes.get(marker_length) != Some(&marker)
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

        let after_second = second
            .use_source(&result, MergeHunkSource::Right)
            .expect("resolve second identical block first");
        assert!(first.is_unresolved(&after_second));
        assert!(second.is_unresolved(&after_second));
        assert_eq!(
            first
                .use_source(&after_second, MergeHunkSource::Left)
                .unwrap(),
            "top\nleft\nmiddle\nright\nbottom\n"
        );

        let after_first = first
            .use_source(&result, MergeHunkSource::Left)
            .expect("resolve first identical block first");
        assert!(first.is_unresolved(&after_first));
        assert!(second.is_unresolved(&after_first));
        let resolved = second
            .use_source(&after_first, MergeHunkSource::Right)
            .expect("the remaining identical block must stay actionable");
        assert_eq!(resolved, "top\nleft\nmiddle\nright\nbottom\n");
        assert!(!first.is_unresolved(&resolved));
        assert!(!second.is_unresolved(&resolved));
    }

    #[test]
    fn a_resolved_hunk_switches_sides_until_the_result_is_edited_by_hand() {
        let hunk = |index: u32, raw: &str, side: &str| MergeEditorHunk {
            index,
            occurrence: 0,
            raw: raw.to_owned(),
            left: format!("{side} left\n"),
            base: format!("{side} base\n"),
            right: format!("{side} right\n"),
        };
        let (one, two) = ("<<<<<<< 1\nx\n>>>>>>> 1\n", "<<<<<<< 2\ny\n>>>>>>> 2\n");
        let hunks = [hunk(0, one, "one"), hunk(1, two, "two")];
        let original = format!("top\n{one}middle\n{two}");
        let pick = |index, source| MergeHunkPick { index, source };

        let first = pick_merge_hunk(
            &original,
            &hunks,
            &[],
            &original,
            pick(0, MergeHunkSource::Left),
        )
        .unwrap();
        let picks = [pick(0, MergeHunkSource::Left)];
        let both = pick_merge_hunk(
            &original,
            &hunks,
            &picks,
            &first,
            pick(1, MergeHunkSource::Right),
        )
        .unwrap();
        let picks = [
            pick(0, MergeHunkSource::Left),
            pick(1, MergeHunkSource::Right),
        ];
        let switched = pick_merge_hunk(
            &original,
            &hunks,
            &picks,
            &both,
            pick(0, MergeHunkSource::Base),
        )
        .unwrap();
        assert_eq!(switched, "top\none base\nmiddle\ntwo right\n");

        let edited = format!("// edited\n{first}");
        assert!(
            pick_merge_hunk(
                &original,
                &hunks,
                &picks[..1],
                &edited,
                pick(0, MergeHunkSource::Right)
            )
            .is_err()
        );
        assert_eq!(
            pick_merge_hunk(
                &original,
                &hunks,
                &picks[..1],
                &edited,
                pick(1, MergeHunkSource::Right)
            )
            .unwrap(),
            "// edited\ntop\none left\nmiddle\ntwo right\n",
            "an unresolved hunk still resolves in place after a hand edit"
        );
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

        let diff = hunk.display_diff("sample.rs", &result);
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
