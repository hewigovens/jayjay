use std::path::Path;

use crate::diff::{DiffSpanStyle, FileDiff, collapse_context_with_mapping, compute_file_diff_full};
use crate::{
    CoreResult, DiffEditFileSelection, DiffEditRange, DiffHunk, FileDiffStats, HunkType,
    diff::DisplayLineMapping,
};

use super::scan::{ScannedExternalDiff, scan_external_diff};

#[derive(Clone, Debug)]
pub struct ExternalDiffFile {
    pub hunk: DiffHunk,
    pub topology_group: Option<String>,
    pub display_diff: FileDiff,
    pub display_to_full: Vec<DisplayLineMapping>,
    pub changed_lines: Vec<u32>,
    pub supports_editing: bool,
    pub old_exists: bool,
    pub new_exists: bool,
    pub old_executable: Option<bool>,
    pub new_executable: Option<bool>,
    pub stats: FileDiffStats,
}

#[derive(Clone, Debug)]
pub struct ExternalDiffSelection {
    pub file: DiffEditFileSelection,
    pub selected_exists: bool,
    pub selected_executable: Option<bool>,
    pub whole_file_side: Option<ExternalDiffSide>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExternalDiffSide {
    Old,
    New,
}

pub fn load_external_diff(
    left: &Path,
    right: &Path,
    editable: bool,
) -> CoreResult<Vec<ExternalDiffFile>> {
    let mut files: Vec<_> = scan_external_diff(left, right, editable)?
        .into_iter()
        .map(prepare_file)
        .collect();
    let paths: Vec<_> = files.iter().map(|file| file.hunk.path.clone()).collect();
    for file in &mut files {
        file.topology_group = topology_group(&paths, &file.hunk.path);
    }
    Ok(files)
}

pub fn diff_edit_ranges(mut lines: Vec<u32>) -> Vec<DiffEditRange> {
    lines.sort_unstable();
    lines.dedup();
    let mut ranges: Vec<DiffEditRange> = Vec::new();
    for line in lines {
        match ranges.last_mut() {
            Some(range) if range.end_line.checked_add(1) == Some(line) => range.end_line = line,
            _ => ranges.push(DiffEditRange {
                start_line: line,
                end_line: line,
            }),
        }
    }
    ranges
}

fn prepare_file(scanned: ScannedExternalDiff) -> ExternalDiffFile {
    let ScannedExternalDiff {
        hunk,
        old_exists,
        new_exists,
        old_is_regular_file,
        new_is_regular_file,
        old_is_text,
        new_is_text,
        old_executable,
        new_executable,
    } = scanned;
    let old = hunk.old.content.as_deref().unwrap_or_default();
    let new = hunk.new.content.as_deref().unwrap_or_default();
    let full_diff = compute_file_diff_full(&hunk.path, old, new, false);
    let insertions = changed_line_count(&full_diff, DiffSpanStyle::Added);
    let deletions = changed_line_count(&full_diff, DiffSpanStyle::Removed);
    let changed_lines: Vec<u32> = full_diff
        .lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| line.is_changed().then_some(index as u32 + 1))
        .collect();
    let mut collapsed = collapse_context_with_mapping(&full_diff);
    collapsed.display_to_full.retain(|mapping| {
        full_diff
            .lines
            .get(mapping.full_line.saturating_sub(1) as usize)
            .is_some_and(|line| line.is_changed())
    });
    let executable_changed = matches!(
        (old_executable, new_executable),
        (Some(old), Some(new)) if old != new
    );
    let existence_changed = old_exists != new_exists;
    let supports_editing = hunk.old.preview.is_none()
        && hunk.new.preview.is_none()
        && old_is_regular_file
        && new_is_regular_file
        && (!changed_lines.is_empty() || executable_changed || existence_changed)
        && hunk.hunk_type != HunkType::Renamed
        && old_is_text
        && new_is_text;
    let stats = FileDiffStats {
        path: hunk.path.clone(),
        insertions,
        deletions,
    };
    ExternalDiffFile {
        hunk,
        topology_group: None,
        display_diff: collapsed.diff,
        display_to_full: collapsed.display_to_full,
        changed_lines,
        supports_editing,
        old_exists,
        new_exists,
        old_executable,
        new_executable,
        stats,
    }
}

fn topology_group(paths: &[String], selected: &str) -> Option<String> {
    let selected_path = Path::new(selected);
    let mut related = paths.iter().filter(|candidate| {
        candidate.as_str() != selected
            && (selected_path.starts_with(Path::new(candidate))
                || Path::new(candidate).starts_with(selected_path))
    });
    let first = related.next()?;
    std::iter::once(selected)
        .chain(std::iter::once(first.as_str()))
        .chain(related.map(String::as_str))
        .min_by_key(|path| Path::new(path).components().count())
        .map(str::to_owned)
}

fn changed_line_count(diff: &FileDiff, style: DiffSpanStyle) -> u32 {
    diff.lines.iter().filter(|line| line.style == style).count() as u32
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{DiffContent, HunkType};

    use super::{
        ScannedExternalDiff, diff_edit_ranges, load_external_diff, prepare_file, topology_group,
    };

    fn scanned(hunk: crate::DiffHunk) -> ScannedExternalDiff {
        let old_exists = hunk.old.content.is_some() || hunk.old.preview.is_some();
        let new_exists = hunk.new.content.is_some() || hunk.new.preview.is_some();
        ScannedExternalDiff {
            hunk,
            old_exists,
            new_exists,
            old_is_regular_file: true,
            new_is_regular_file: true,
            old_is_text: true,
            new_is_text: true,
            old_executable: None,
            new_executable: None,
        }
    }

    #[test]
    fn coalesces_sorted_unique_line_ranges() {
        let ranges = diff_edit_ranges(vec![5, 2, 3, 3, 9]);
        assert_eq!(
            ranges
                .iter()
                .map(|range| (range.start_line, range.end_line))
                .collect::<Vec<_>>(),
            vec![(2, 3), (5, 5), (9, 9)]
        );
    }

    #[test]
    fn groups_paths_that_cannot_coexist_in_one_tree() {
        let paths = vec![
            "item".to_owned(),
            "item/child.txt".to_owned(),
            "other.txt".to_owned(),
        ];

        assert_eq!(topology_group(&paths, "item"), Some("item".to_owned()));
        assert_eq!(
            topology_group(&paths, "item/child.txt"),
            Some("item".to_owned())
        );
        assert_eq!(topology_group(&paths, "other.txt"), None);
    }

    #[test]
    fn selection_mapping_excludes_unchanged_context() {
        let unchanged = (1..=10)
            .map(|line| format!("line {line}"))
            .collect::<Vec<_>>();
        let old = unchanged.join("\n") + "\n";
        let mut new = unchanged;
        new[4] = "changed".to_owned();
        let file = prepare_file(scanned(crate::DiffHunk {
            path: "file.txt".to_owned(),
            old_path: None,
            old: DiffContent::new(Some(old), None),
            new: DiffContent::new(Some(new.join("\n") + "\n"), None),
            hunk_type: HunkType::Modified,
            supports_conflict_editor: false,
            supports_file_editor: true,
            review_identity: String::new(),
            projection: None,
        }));

        assert_eq!(file.display_to_full.len(), file.changed_lines.len());
        assert!(
            file.display_to_full
                .iter()
                .all(|mapping| file.changed_lines.contains(&mapping.full_line))
        );
    }

    #[test]
    fn empty_file_existence_changes_support_file_selection() {
        let file = prepare_file(scanned(crate::DiffHunk {
            path: "empty.txt".to_owned(),
            old_path: None,
            old: DiffContent::default(),
            new: DiffContent::new(Some(String::new()), None),
            hunk_type: HunkType::Added,
            supports_conflict_editor: false,
            supports_file_editor: true,
            review_identity: String::new(),
            projection: None,
        }));

        assert!(file.supports_editing);
        assert!(!file.old_exists);
        assert!(file.new_exists);
    }

    #[test]
    fn placeholder_prefixed_regular_text_supports_line_selection() {
        let left = tempfile::tempdir().expect("left");
        let right = tempfile::tempdir().expect("right");
        fs::write(left.path().join("literal.txt"), "symlink -> old\n").expect("left text");
        fs::write(right.path().join("literal.txt"), "symlink -> new\n").expect("right text");
        let files = load_external_diff(left.path(), right.path(), true).expect("load diff");
        let file = files.first().expect("changed file");

        assert!(file.supports_editing);
        assert!(!file.changed_lines.is_empty());
    }

    #[test]
    fn executable_only_changes_support_file_selection() {
        let file = prepare_file(ScannedExternalDiff {
            hunk: crate::DiffHunk {
                path: "script.sh".to_owned(),
                old_path: None,
                old: DiffContent::new(Some("#!/bin/sh\n".to_owned()), None),
                new: DiffContent::new(Some("#!/bin/sh\n".to_owned()), None),
                hunk_type: HunkType::Modified,
                supports_conflict_editor: false,
                supports_file_editor: true,
                review_identity: String::new(),
                projection: None,
            },
            old_exists: true,
            new_exists: true,
            old_is_regular_file: true,
            new_is_regular_file: true,
            old_is_text: true,
            new_is_text: true,
            old_executable: Some(false),
            new_executable: Some(true),
        });

        assert!(file.changed_lines.is_empty());
        assert!(file.supports_editing);
    }
}
