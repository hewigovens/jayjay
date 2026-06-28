use std::collections::HashMap;

use jayjay_primitives::{
    NoteEntry, NoteSide, NoteStatus, ReviewDiffProvider, ReviewFileDiff, ReviewHunk,
    ReviewNoteStatus, ReviewResult,
};
use jj_diff::{
    DiffLine, DiffSide, build_diff_display_lines, change_group_for_anchor, compute_file_diff,
};

use crate::store::ReviewStore;

/// Per-reconcile caches: raw contents once per file, rendered display lines once per (file, whitespace-mode) — the diff pipeline is expensive and several notes often share a file.
#[derive(Default)]
struct ReconcileCache {
    contents: HashMap<String, Option<ReviewFileDiff>>,
    display_lines: HashMap<(String, bool), Option<Vec<DiffLine>>>,
}

impl ReviewStore {
    pub fn reconcile(
        &self,
        provider: &impl ReviewDiffProvider,
        change_id: &str,
        include_resolved: bool,
    ) -> ReviewResult<Vec<ReviewNoteStatus>> {
        let notes = self.list_notes(change_id, include_resolved);
        if notes.is_empty() {
            return Ok(vec![]);
        }

        let hunks: HashMap<String, ReviewHunk> = provider
            .review_hunks()?
            .into_iter()
            .map(|hunk| (hunk.path.clone(), hunk))
            .collect();
        let mut cache = ReconcileCache::default();

        Ok(notes
            .into_iter()
            .map(|note| status_for_note(&note, provider, &hunks, &mut cache))
            .collect())
    }
}

fn status_for_note(
    note: &NoteEntry,
    provider: &impl ReviewDiffProvider,
    hunks: &HashMap<String, ReviewHunk>,
    cache: &mut ReconcileCache,
) -> ReviewNoteStatus {
    if note.resolved {
        return ReviewNoteStatus::new(note, NoteStatus::Resolved, None);
    }
    let Some(hunk) = hunks.get(&note.path) else {
        return ReviewNoteStatus::new(note, NoteStatus::Orphaned, None);
    };
    if hunk.review_identity != note.identity {
        return ReviewNoteStatus::new(note, NoteStatus::Stale, None);
    }

    let Some(lines) = display_lines(note, provider, hunk, cache) else {
        return ReviewNoteStatus::new(note, NoteStatus::Stale, None);
    };
    let group =
        change_group_for_anchor(lines, diff_side(note.side), note.line, &note.anchor_excerpt);
    match group {
        Some(group) => ReviewNoteStatus::new(note, NoteStatus::Current, Some(group.index)),
        None => ReviewNoteStatus::new(note, NoteStatus::Stale, None),
    }
}

/// Replays the exact pipeline the GUI anchors against — Histogram line diff with collapsed context and the note's recorded whitespace mode, then conflict display lines — so a note created in the app resolves to the same change group here and in the CLI.
fn display_lines<'a>(
    note: &NoteEntry,
    provider: &impl ReviewDiffProvider,
    hunk: &ReviewHunk,
    cache: &'a mut ReconcileCache,
) -> Option<&'a [DiffLine]> {
    let key = (note.path.clone(), note.ignore_whitespace);
    if !cache.display_lines.contains_key(&key) {
        let contents = cache
            .contents
            .entry(note.path.clone())
            .or_insert_with(|| provider.review_file_diff(hunk).ok());
        let lines = contents.as_ref().map(|file_diff| {
            let diff = compute_file_diff(
                &note.path,
                file_diff.old_content.as_deref().unwrap_or(""),
                file_diff.new_content.as_deref().unwrap_or(""),
                note.ignore_whitespace,
            );
            build_diff_display_lines(&diff.lines)
        });
        cache.display_lines.insert(key.clone(), lines);
    }
    cache.display_lines[&key].as_deref()
}

fn diff_side(side: NoteSide) -> DiffSide {
    match side {
        NoteSide::Old => DiffSide::Old,
        NoteSide::New => DiffSide::New,
    }
}

#[cfg(test)]
mod tests {
    use jayjay_primitives::HunkType;

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
        let mut cache = ReconcileCache::default();
        let lines = display_lines(note, &provider, &hunk, &mut cache)?;
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
        // The anchor must resolve to the same group index the GUI computes via jj_diff::change_groups over display lines, for every group.
        let old = "a\nb\nc\nd\ne\nf\ng\nh\ni\nj\nk\nl\nm\nn\no\np\n";
        let new = "a\nX\nc\nd\ne\nf\ng\nh\nY\nj\nk\nl\nm\nn\no\nZ\n";
        let diff = compute_file_diff("a.txt", old, new, false);
        let lines = build_diff_display_lines(&diff.lines);
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
        // Whitespace-insensitive pairing can attribute a different duplicate line as "added" (iw=true pairs new line 1 "foo " with old "foo" and marks line 2 added; iw=false marks line 1 added), so reconcile must replay the note's recorded mode or the note the user can see anchored in the app reports Stale.
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
        // Inserting a copy of an existing block is where line-diff algorithms disagree about which duplicate is "added"; the anchor was produced by the GUI's Histogram pipeline, so reconciling with the same pipeline must find it regardless of which copy the algorithm attributes.
        let old = "fn a() {}\nfn b() {}\nfn c() {}\n";
        let new = "fn a() {}\nfn b() {}\nfn a() {}\nfn b() {}\nfn c() {}\n";
        let diff = compute_file_diff("a.rs", old, new, false);
        let lines = build_diff_display_lines(&diff.lines);
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
