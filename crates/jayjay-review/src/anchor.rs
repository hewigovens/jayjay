use jayjay_primitives::ReviewFileDiff;
use jayjay_primitives::{NoteAnchor, NoteSide, ReviewDiffProvider, ReviewResult};
use jj_diff::{ChangeGroup, DiffLine, DiffSide, DiffSpanStyle, change_group_for_anchor};

use crate::replay::{diff_side, render_display_lines};

/// Builds the anchor a GUI note would carry for (path, side, line), replaying the same Histogram pipeline reconcile uses so a CLI-created note reports Current on every surface. `None` when the line is not a changed line of the provider's diff.
pub fn build_note_anchor(
    provider: &impl ReviewDiffProvider,
    change_id: &str,
    path: &str,
    side: NoteSide,
    line: u32,
) -> ReviewResult<Option<NoteAnchor>> {
    let Some(hunk) = provider
        .review_hunks()?
        .into_iter()
        .find(|hunk| hunk.path == path)
    else {
        return Ok(None);
    };
    let file_diff = provider.review_file_diff(&hunk)?;
    let lines = render_display_lines(path, &file_diff, false);
    let Some((excerpt, group)) = changed_line_group(&lines, side, line) else {
        return Ok(None);
    };
    Ok(Some(NoteAnchor {
        change_id: change_id.to_owned(),
        path: path.to_owned(),
        identity: hunk.review_identity,
        side,
        line,
        anchor_excerpt: excerpt,
        anchor_context: group.anchor_context,
        ignore_whitespace: false,
    }))
}

/// Index of the exact-whitespace display change group containing the changed line, or `None` when the line is not a changed line.
pub fn change_group_index(
    path: &str,
    file_diff: &ReviewFileDiff,
    side: NoteSide,
    line: u32,
) -> Option<u32> {
    let lines = render_display_lines(path, file_diff, false);
    changed_line_group(&lines, side, line).map(|(_, group)| group.index)
}

fn changed_line_group(
    lines: &[DiffLine],
    side: NoteSide,
    line: u32,
) -> Option<(String, ChangeGroup)> {
    let anchor_side = diff_side(side);
    let display_line = lines.iter().find(|candidate| match anchor_side {
        DiffSide::New => {
            candidate.style == DiffSpanStyle::Added && candidate.new_line_no == Some(line)
        }
        DiffSide::Old => {
            candidate.style == DiffSpanStyle::Removed && candidate.old_line_no == Some(line)
        }
    })?;
    let excerpt: String = display_line
        .spans
        .iter()
        .map(|span| span.text.as_str())
        .collect();
    let group = change_group_for_anchor(lines, anchor_side, line, &excerpt)?;
    Some((excerpt, group))
}

#[cfg(test)]
mod tests {
    use jayjay_primitives::{HunkType, ReviewHunk};

    use super::*;

    struct OneFile;

    impl ReviewDiffProvider for OneFile {
        fn review_hunks(&self) -> ReviewResult<Vec<ReviewHunk>> {
            Ok(vec![ReviewHunk {
                path: "a.txt".to_owned(),
                old_path: None,
                hunk_type: HunkType::Modified,
                review_identity: "id-v1".to_owned(),
            }])
        }

        fn review_file_diff(&self, _hunk: &ReviewHunk) -> ReviewResult<ReviewFileDiff> {
            Ok(ReviewFileDiff {
                old_content: Some("a\nb\n".to_owned()),
                new_content: Some("a\nchanged\nb\n".to_owned()),
            })
        }
    }

    #[test]
    fn builds_anchor_that_reconciles_current() {
        let anchor = build_note_anchor(&OneFile, "c1", "a.txt", NoteSide::New, 2)
            .unwrap()
            .expect("changed line must anchor");
        assert_eq!(anchor.identity, "id-v1");
        assert_eq!(anchor.anchor_excerpt, "changed");
        assert!(!anchor.anchor_context.is_empty());

        // The produced anchor must resolve to a change group through the same lookup reconcile performs.
        let file_diff = ReviewFileDiff {
            old_content: Some("a\nb\n".to_owned()),
            new_content: Some("a\nchanged\nb\n".to_owned()),
        };
        let lines = render_display_lines("a.txt", &file_diff, false);
        assert!(
            change_group_for_anchor(&lines, DiffSide::New, anchor.line, &anchor.anchor_excerpt)
                .is_some()
        );
    }

    #[test]
    fn change_group_index_follows_the_anchor_group() {
        let file_diff = ReviewFileDiff {
            old_content: Some("a\nb\nc\nd\ne\nf\ng\nh\ni\nj\n".to_owned()),
            new_content: Some("a\nX\nc\nd\ne\nf\ng\nh\nY\nj\n".to_owned()),
        };
        assert_eq!(
            change_group_index("a.txt", &file_diff, NoteSide::New, 2),
            Some(0)
        );
        assert_eq!(
            change_group_index("a.txt", &file_diff, NoteSide::Old, 9),
            Some(1)
        );
        assert_eq!(
            change_group_index("a.txt", &file_diff, NoteSide::New, 3),
            None
        );
    }

    #[test]
    fn unchanged_line_and_unknown_file_yield_none() {
        assert!(
            build_note_anchor(&OneFile, "c1", "a.txt", NoteSide::New, 1)
                .unwrap()
                .is_none()
        );
        assert!(
            build_note_anchor(&OneFile, "c1", "a.txt", NoteSide::Old, 1)
                .unwrap()
                .is_none()
        );
        assert!(
            build_note_anchor(&OneFile, "c1", "missing.txt", NoteSide::New, 1)
                .unwrap()
                .is_none()
        );
    }
}
