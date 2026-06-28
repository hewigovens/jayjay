use std::collections::HashMap;

use jayjay_primitives::{NoteEntry, NoteSide, ReviewDiffProvider, ReviewFileDiff, ReviewHunk};
use jj_diff::{DiffLine, DiffSide, build_diff_display_lines, compute_file_diff};

/// Replays the GUI's exact diff pipeline so a note anchored in the app resolves to the same change group in the CLI.
pub(crate) fn render_display_lines(
    path: &str,
    file_diff: &ReviewFileDiff,
    ignore_whitespace: bool,
) -> Vec<DiffLine> {
    let diff = compute_file_diff(
        path,
        file_diff.old_content.as_deref().unwrap_or(""),
        file_diff.new_content.as_deref().unwrap_or(""),
        ignore_whitespace,
    );
    build_diff_display_lines(&diff.lines)
}

/// Per-reconcile cache: the diff pipeline is expensive and several notes often share a file.
#[derive(Default)]
pub(crate) struct ReplayCache {
    contents: HashMap<String, Option<ReviewFileDiff>>,
    display_lines: HashMap<(String, bool), Option<Vec<DiffLine>>>,
}

impl ReplayCache {
    /// Display lines rendered with the note's recorded whitespace mode; `None` when the provider cannot produce the diff.
    pub(crate) fn display_lines(
        &mut self,
        note: &NoteEntry,
        provider: &impl ReviewDiffProvider,
        hunk: &ReviewHunk,
    ) -> Option<&[DiffLine]> {
        let key = (note.path.clone(), note.ignore_whitespace);
        if !self.display_lines.contains_key(&key) {
            let contents = self
                .contents
                .entry(note.path.clone())
                .or_insert_with(|| provider.review_file_diff(hunk).ok());
            let lines = contents.as_ref().map(|file_diff| {
                render_display_lines(&note.path, file_diff, note.ignore_whitespace)
            });
            self.display_lines.insert(key.clone(), lines);
        }
        self.display_lines[&key].as_deref()
    }
}

pub(crate) fn diff_side(side: NoteSide) -> DiffSide {
    match side {
        NoteSide::Old => DiffSide::Old,
        NoteSide::New => DiffSide::New,
    }
}

#[cfg(test)]
mod tests {
    use jayjay_primitives::{HunkType, ReviewResult};
    use jj_diff::change_group_for_anchor;

    use super::*;

    struct FixedDiff(ReviewFileDiff);

    impl ReviewDiffProvider for FixedDiff {
        fn review_hunks(&self) -> ReviewResult<Vec<ReviewHunk>> {
            Ok(vec![])
        }

        fn review_file_diff(&self, _hunk: &ReviewHunk) -> ReviewResult<ReviewFileDiff> {
            Ok(ReviewFileDiff {
                old_content: self.0.old_content.clone(),
                new_content: self.0.new_content.clone(),
            })
        }
    }

    fn anchor_group_index(note: &NoteEntry, file_diff: &ReviewFileDiff) -> Option<u32> {
        let provider = FixedDiff(ReviewFileDiff {
            old_content: file_diff.old_content.clone(),
            new_content: file_diff.new_content.clone(),
        });
        let hunk = ReviewHunk {
            path: note.path.clone(),
            old_path: None,
            hunk_type: HunkType::Modified,
            review_identity: note.identity.clone(),
        };
        let mut cache = ReplayCache::default();
        let lines = cache.display_lines(note, &provider, &hunk)?;
        change_group_for_anchor(lines, diff_side(note.side), note.line, &note.anchor_excerpt)
            .map(|group| group.index)
    }

    fn note(side: NoteSide, line: u32, excerpt: &str) -> NoteEntry {
        NoteEntry {
            id: "n1".to_owned(),
            change_id: "c1".to_owned(),
            path: "a.txt".to_owned(),
            identity: "id".to_owned(),
            side,
            line,
            anchor_excerpt: excerpt.to_owned(),
            anchor_context: vec![],
            ignore_whitespace: false,
            body: "check".to_owned(),
            created_at_ms: 1,
            updated_at_ms: 1,
            resolved: false,
            resolved_at_ms: None,
        }
    }

    fn file_diff(old: &str, new: &str) -> ReviewFileDiff {
        ReviewFileDiff {
            old_content: Some(old.to_owned()),
            new_content: Some(new.to_owned()),
        }
    }

    #[test]
    fn finds_anchor_group() {
        let group = anchor_group_index(
            &note(NoteSide::New, 2, "changed"),
            &file_diff("a\nb\n", "a\nchanged\nb\n"),
        );
        assert_eq!(group, Some(0));
    }

    #[test]
    fn missing_anchor_is_stale() {
        let group = anchor_group_index(
            &note(NoteSide::New, 2, "other"),
            &file_diff("a\nb\n", "a\nchanged\nb\n"),
        );
        assert_eq!(group, None);
    }

    #[test]
    fn anchor_group_matches_canonical_change_groups() {
        // Every anchor must resolve to the group index the GUI computes via jj_diff::change_groups.
        let old = "a\nb\nc\nd\ne\nf\ng\nh\ni\nj\nk\nl\nm\nn\no\np\n";
        let new = "a\nX\nc\nd\ne\nf\ng\nh\nY\nj\nk\nl\nm\nn\no\nZ\n";
        let lines = render_display_lines("a.txt", &file_diff(old, new), false);
        let groups = jj_diff::change_groups(&lines);
        assert!(groups.len() > 1, "test needs multiple change groups");

        for group in groups {
            let side = match group.anchor_side {
                DiffSide::Old => NoteSide::Old,
                DiffSide::New => NoteSide::New,
            };
            let resolved = anchor_group_index(
                &note(side, group.anchor_line, &group.anchor_excerpt),
                &file_diff(old, new),
            );
            assert_eq!(resolved, Some(group.index));
        }
    }

    #[test]
    fn anchor_created_under_ignore_whitespace_reconciles_with_that_mode() {
        // The two whitespace modes attribute different duplicate lines as "added", so reconcile must replay the note's recorded mode or a note visibly anchored in the app reports Stale.
        let old = "foo\nbar\n";
        let new = "foo \nfoo\nbar\n";

        let mut ws_note = note(NoteSide::New, 2, "foo");
        ws_note.ignore_whitespace = true;
        assert_eq!(anchor_group_index(&ws_note, &file_diff(old, new)), Some(0));

        ws_note.ignore_whitespace = false;
        assert_eq!(anchor_group_index(&ws_note, &file_diff(old, new)), None);
    }

    #[test]
    fn duplicate_block_anchor_reconciles_current() {
        // Diff algorithms disagree about which duplicate copy is "added"; replaying the GUI's pipeline must find the anchor either way.
        let old = "fn a() {}\nfn b() {}\nfn c() {}\n";
        let new = "fn a() {}\nfn b() {}\nfn a() {}\nfn b() {}\nfn c() {}\n";
        let lines = render_display_lines("a.rs", &file_diff(old, new), false);
        let group = jj_diff::change_groups(&lines)
            .into_iter()
            .next()
            .expect("change group");
        let side = match group.anchor_side {
            DiffSide::Old => NoteSide::Old,
            DiffSide::New => NoteSide::New,
        };

        let resolved = anchor_group_index(
            &note(side, group.anchor_line, &group.anchor_excerpt),
            &file_diff(old, new),
        );

        assert_eq!(resolved, Some(group.index));
    }
}
