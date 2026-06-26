use std::collections::HashSet;
use std::path::Path;

use crate::hash::hex_sha256;
use crate::types::*;

/// Detect renames by matching removed+added files via content similarity or filename similarity.
pub(super) fn detect_renames(hunks: &mut Vec<DiffHunk>) {
    let removed_indices: Vec<usize> = hunks
        .iter()
        .enumerate()
        .filter(|(_, h)| h.hunk_type == HunkType::Removed)
        .map(|(i, _)| i)
        .collect();
    let added_indices: Vec<usize> = hunks
        .iter()
        .enumerate()
        .filter(|(_, h)| h.hunk_type == HunkType::Added)
        .map(|(i, _)| i)
        .collect();

    let mut matched_removed = Vec::new();
    let mut matched_added = Vec::new();

    for &removed_index in &removed_indices {
        let mut best_match: Option<(usize, f64)> = None;

        for &added_index in &added_indices {
            if matched_added.contains(&added_index) {
                continue;
            }
            let score = rename_score(&hunks[removed_index], &hunks[added_index]);
            if score > 0.5 && !best_match.is_some_and(|(_, best_score)| score <= best_score) {
                best_match = Some((added_index, score));
            }
        }

        if let Some((added_index, _score)) = best_match {
            let old_path = hunks[removed_index].path.clone();
            let removed_preview = hunks[removed_index].old_preview.clone();
            // Combine both sides so removed-side changes also invalidate the mark.
            let combined_identity = hex_sha256(
                format!(
                    "rename|{}|{}",
                    hunks[removed_index].review_identity, hunks[added_index].review_identity
                )
                .as_bytes(),
            );
            // Only a byte-equal pair is a pure rename; set-similarity scores 1.0 for
            // reordered or duplicate-only content, so never clear contents on score alone.
            let byte_equal = hunks[removed_index].old_content == hunks[added_index].new_content;

            hunks[added_index].old_path = Some(old_path);
            hunks[added_index].hunk_type = HunkType::Renamed;
            hunks[added_index].old_preview = removed_preview;
            hunks[added_index].review_identity = combined_identity;

            if byte_equal {
                hunks[added_index].old_content = None;
                hunks[added_index].new_content = None;
            } else {
                hunks[added_index].old_content = hunks[removed_index].old_content.clone();
            }

            matched_removed.push(removed_index);
            matched_added.push(added_index);
        }
    }

    matched_removed.sort_unstable();
    for &index in matched_removed.iter().rev() {
        hunks.remove(index);
    }
}

/// Score how likely a removed+added pair is a rename. Returns 0.0–1.0.
fn rename_score(removed: &DiffHunk, added: &DiffHunk) -> f64 {
    let old_path = Path::new(&removed.path);
    let new_path = Path::new(&added.path);
    let old_name = old_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let new_name = new_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    let old_content = removed.old_content.as_deref().unwrap_or("");
    let new_content = added.new_content.as_deref().unwrap_or("");
    let has_content = !old_content.is_empty() || !new_content.is_empty();

    if has_content && old_content == new_content {
        return 1.0;
    }

    if !old_name.is_empty() && old_name.eq_ignore_ascii_case(new_name) {
        // Filename alone is a strong signal; content only refines the score.
        let content_sim = if has_content {
            content_similarity(old_content, new_content)
        } else {
            0.0
        };
        return 0.6 + content_sim * 0.4;
    }

    // Extension-only match requires real content — empty strings aren't evidence.
    if !has_content {
        return 0.0;
    }

    let old_ext = old_path.extension().and_then(|e| e.to_str());
    let new_ext = new_path.extension().and_then(|e| e.to_str());
    if old_ext == new_ext && old_ext.is_some() {
        let similarity = content_similarity(old_content, new_content);
        if similarity > 0.7 {
            return similarity;
        }
    }

    0.0
}

/// Rough content similarity: ratio of matching lines.
fn content_similarity(a: &str, b: &str) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let a_lines: HashSet<&str> = a.lines().collect();
    let b_lines: HashSet<&str> = b.lines().collect();
    let intersection = a_lines.intersection(&b_lines).count();
    let union = a_lines.union(&b_lines).count();
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hunk(path: &str, hunk_type: HunkType, old: Option<&str>, new: Option<&str>) -> DiffHunk {
        hunk_with_identity(path, hunk_type, old, new, "")
    }

    fn hunk_with_identity(
        path: &str,
        hunk_type: HunkType,
        old: Option<&str>,
        new: Option<&str>,
        review_identity: &str,
    ) -> DiffHunk {
        DiffHunk {
            path: path.to_owned(),
            old_path: None,
            old_content: old.map(|s| s.to_owned()),
            new_content: new.map(|s| s.to_owned()),
            old_preview: None,
            new_preview: None,
            hunk_type,
            review_identity: review_identity.to_owned(),
        }
    }

    #[test]
    fn content_similarity_both_empty_is_identical() {
        assert_eq!(content_similarity("", ""), 1.0);
    }

    #[test]
    fn content_similarity_identical() {
        assert_eq!(content_similarity("a\nb\n", "a\nb\n"), 1.0);
    }

    #[test]
    fn content_similarity_disjoint() {
        assert_eq!(content_similarity("a\n", "z\n"), 0.0);
    }

    #[test]
    fn rename_detected_with_content() {
        let mut hunks = vec![
            hunk("old.rs", HunkType::Removed, Some("fn main() {}"), None),
            hunk("new.rs", HunkType::Added, None, Some("fn main() {}")),
        ];
        detect_renames(&mut hunks);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].hunk_type, HunkType::Renamed);
        assert_eq!(hunks[0].path, "new.rs");
        assert_eq!(hunks[0].old_path.as_deref(), Some("old.rs"));
    }

    #[test]
    fn rename_with_reordered_content_keeps_diff() {
        // Regression: same-basename rename whose lines are reordered scores 1.0 via
        // set-similarity, but the contents are NOT byte-equal, so the diff must survive.
        let mut hunks = vec![
            hunk("a/x.rs", HunkType::Removed, Some("a\nb\nc\n"), None),
            hunk("b/x.rs", HunkType::Added, None, Some("c\nb\na\n")),
        ];
        detect_renames(&mut hunks);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].hunk_type, HunkType::Renamed);
        assert_eq!(
            hunks[0].old_content.as_deref(),
            Some("a\nb\nc\n"),
            "reordered rename must keep the before content"
        );
        assert_eq!(
            hunks[0].new_content.as_deref(),
            Some("c\nb\na\n"),
            "reordered rename must keep the after content"
        );
    }

    #[test]
    fn rename_with_duplicate_only_change_keeps_diff() {
        // Removing one of two identical lines is set-equal (score 1.0) but not byte-equal.
        let mut hunks = vec![
            hunk("a/y.rs", HunkType::Removed, Some("x\nx\ny\n"), None),
            hunk("b/y.rs", HunkType::Added, None, Some("x\ny\n")),
        ];
        detect_renames(&mut hunks);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].hunk_type, HunkType::Renamed);
        assert_eq!(hunks[0].new_content.as_deref(), Some("x\ny\n"));
    }

    #[test]
    fn byte_equal_rename_clears_content() {
        // A true pure rename (byte-equal contents) still folds into a content-free hunk.
        let mut hunks = vec![
            hunk("a/z.rs", HunkType::Removed, Some("same\n"), None),
            hunk("b/z.rs", HunkType::Added, None, Some("same\n")),
        ];
        detect_renames(&mut hunks);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].hunk_type, HunkType::Renamed);
        assert!(hunks[0].old_content.is_none());
        assert!(hunks[0].new_content.is_none());
    }

    #[test]
    fn rename_review_identity_combines_both_sides() {
        // Folded rename's identity must reflect both sides, not just the added one.
        let mut hunks = vec![
            hunk_with_identity("old.rs", HunkType::Removed, Some("body"), None, "id-old-v1"),
            hunk_with_identity("new.rs", HunkType::Added, None, Some("body"), "id-new"),
        ];
        detect_renames(&mut hunks);
        let renamed_v1 = hunks[0].review_identity.clone();

        let mut hunks_v2 = vec![
            hunk_with_identity("old.rs", HunkType::Removed, Some("body"), None, "id-old-v2"),
            hunk_with_identity("new.rs", HunkType::Added, None, Some("body"), "id-new"),
        ];
        detect_renames(&mut hunks_v2);
        let renamed_v2 = hunks_v2[0].review_identity.clone();

        assert_ne!(renamed_v1, renamed_v2);
        assert_ne!(renamed_v1, "id-new");
    }

    #[test]
    fn no_rename_different_names_empty_content() {
        // Regression: different basenames with empty content must not match on extension alone.
        let mut hunks = vec![
            hunk("PLAN.md", HunkType::Removed, None, None),
            hunk("Roadmap.md", HunkType::Added, None, None),
        ];
        detect_renames(&mut hunks);
        assert_eq!(hunks.len(), 2);
    }

    #[test]
    fn batch_move_without_content_pairs_by_filename_only() {
        // Regression: batch move with empty content must pair by filename and leave extras unpaired.
        let mut hunks = vec![
            hunk("pkg/DiffColors.swift", HunkType::Added, None, None),
            hunk("pkg/ImageDiffView.swift", HunkType::Added, None, None),
            hunk("pkg/NativeDiffView.swift", HunkType::Added, None, None),
            hunk(
                "pkg/SideBySideDiffRows.swift",
                HunkType::Removed,
                None,
                None,
            ),
            hunk("app/DiffColors.swift", HunkType::Removed, None, None),
            hunk("app/ImageDiffView.swift", HunkType::Removed, None, None),
            hunk("app/NativeDiffView.swift", HunkType::Removed, None, None),
        ];
        detect_renames(&mut hunks);

        // 3 renames + 1 unpaired deletion = 4 entries.
        assert_eq!(hunks.len(), 4);

        let rename_pairs: Vec<(&str, Option<&str>)> = hunks
            .iter()
            .filter(|h| h.hunk_type == HunkType::Renamed)
            .map(|h| (h.path.as_str(), h.old_path.as_deref()))
            .collect();
        assert_eq!(rename_pairs.len(), 3);
        assert!(rename_pairs.contains(&("pkg/DiffColors.swift", Some("app/DiffColors.swift"))));
        assert!(
            rename_pairs.contains(&("pkg/ImageDiffView.swift", Some("app/ImageDiffView.swift")))
        );
        assert!(
            rename_pairs.contains(&("pkg/NativeDiffView.swift", Some("app/NativeDiffView.swift")))
        );

        let orphaned_delete = hunks
            .iter()
            .find(|h| h.hunk_type == HunkType::Removed)
            .expect("SideBySideDiffRows should remain as a pure deletion");
        assert_eq!(orphaned_delete.path, "pkg/SideBySideDiffRows.swift");
    }

    #[test]
    fn rename_same_filename_different_dir_no_content() {
        let mut hunks = vec![
            hunk("src/lib.rs", HunkType::Removed, None, None),
            hunk("core/lib.rs", HunkType::Added, None, None),
        ];
        detect_renames(&mut hunks);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].hunk_type, HunkType::Renamed);
    }

    #[test]
    fn no_rename_across_different_extensions() {
        let mut hunks = vec![
            hunk("old.rs", HunkType::Removed, None, None),
            hunk("new.py", HunkType::Added, None, None),
        ];
        detect_renames(&mut hunks);
        assert_eq!(hunks.len(), 2, "different extensions should not match");
    }

    #[test]
    fn rename_carries_old_preview_from_removed_hunk() {
        // Regression: the renamed hunk's Before pane was always empty because
        // detect_renames copied old_content but forgot old_preview.
        let mut removed = hunk(
            "old/icon.png",
            HunkType::Removed,
            Some("<image (100 bytes)>"),
            None,
        );
        removed.old_preview = Some(DiffPreview::Image {
            path: "/tmp/jayjay-images/abc123.png".to_owned(),
        });
        let added = hunk(
            "new/icon.png",
            HunkType::Added,
            None,
            Some("<image (100 bytes)>"),
        );

        let mut hunks = vec![removed, added];
        detect_renames(&mut hunks);

        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].hunk_type, HunkType::Renamed);
        assert_eq!(hunks[0].old_path.as_deref(), Some("old/icon.png"));

        let Some(DiffPreview::Image { path }) = hunks[0].old_preview.as_ref() else {
            panic!(
                "renamed hunk should carry the removed side's preview, got {:?}",
                hunks[0].old_preview
            );
        };
        assert_eq!(path, "/tmp/jayjay-images/abc123.png");
    }

    #[test]
    fn content_free_rename_predicate_tracks_detect_renames() {
        // A folded byte-equal rename has no content to diff.
        let mut identical = vec![
            hunk("a/z.rs", HunkType::Removed, Some("same\n"), None),
            hunk("b/z.rs", HunkType::Added, None, Some("same\n")),
        ];
        detect_renames(&mut identical);
        assert!(identical[0].is_content_free_rename());

        // A rename that also changed content keeps its diff and is not content-free.
        let mut changed = vec![
            hunk("a/x.rs", HunkType::Removed, Some("a\n"), None),
            hunk("b/x.rs", HunkType::Added, None, Some("b\n")),
        ];
        detect_renames(&mut changed);
        assert!(!changed[0].is_content_free_rename());
    }
}
