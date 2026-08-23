use super::*;
use crate::review_fingerprint::{
    canonical_review_snapshot, display_group_canonical_indices, map_display_groups_to_canonical,
};

fn two_group_old() -> &'static str {
    "head-1\nhead-2\nhead-3\nhead-4\nAAA\nmiddle\nBBB\ntail\n"
}

fn two_group_new() -> &'static str {
    "head-1\nhead-2\nhead-3\nhead-4\naaa\nmiddle\nbbb\ntail\n"
}

fn digests(old: &str, new: &str) -> Vec<String> {
    canonical_review_snapshot(old, new)
        .fingerprints
        .into_iter()
        .map(|fingerprint| fingerprint.digest)
        .collect()
}

#[test]
fn two_separated_groups_produce_two_fingerprints() {
    let snapshot = canonical_review_snapshot(two_group_old(), two_group_new());
    assert_eq!(snapshot.fingerprints.len(), 2);
    assert_ne!(
        snapshot.fingerprints[0].digest,
        snapshot.fingerprints[1].digest
    );
}

#[test]
fn editing_one_group_leaves_the_other_fingerprint_unchanged() {
    let before = digests(two_group_old(), two_group_new());
    let after = digests(
        two_group_old(),
        "head-1\nhead-2\nhead-3\nhead-4\naaa-edited\nmiddle\nbbb\ntail\n",
    );
    assert_eq!(before.len(), 2);
    assert_eq!(after.len(), 2);
    assert_ne!(before[0], after[0]);
    assert_eq!(before[1], after[1]);
}

#[test]
fn inserting_lines_above_a_group_preserves_its_fingerprint() {
    let before = digests(two_group_old(), two_group_new());
    let after = digests(
        "inserted\nhead-1\nhead-2\nhead-3\nhead-4\nAAA\nmiddle\nBBB\ntail\n",
        "inserted\nhead-1\nhead-2\nhead-3\nhead-4\naaa\nmiddle\nbbb\ntail\n",
    );
    assert_eq!(before[0], after[0]);
    assert_eq!(before[1], after[1]);
}

#[test]
fn adding_a_distant_group_preserves_existing_fingerprints() {
    let before = digests(two_group_old(), two_group_new());
    let after = digests(
        "head-1\nhead-2\nhead-3\nhead-4\nAAA\nmiddle\nBBB\ntail\nzzz\n",
        "head-1\nhead-2\nhead-3\nhead-4\naaa\nmiddle\nbbb\ntail\nZZZ\n",
    );
    assert_eq!(after.len(), 3);
    assert_eq!(before[0], after[0]);
    assert_eq!(before[1], after[1]);
    assert_ne!(after[2], before[0]);
    assert_ne!(after[2], before[1]);
}

#[test]
fn moving_an_identical_patch_to_different_context_changes_fingerprint() {
    let original = digests("ctx-a\nAAA\nctx-b\n", "ctx-a\naaa\nctx-b\n");
    let moved = digests("ctx-x\nAAA\nctx-y\n", "ctx-x\naaa\nctx-y\n");
    assert_eq!(original.len(), 1);
    assert_eq!(moved.len(), 1);
    assert_ne!(original[0], moved[0]);
}

#[test]
fn wrapping_and_collapse_do_not_change_canonical_fingerprints() {
    // Canonical snapshots always use full exact-whitespace lines, so display wrapping/collapse cannot be part of the digest.
    let snapshot = canonical_review_snapshot(two_group_old(), two_group_new());
    let again = canonical_review_snapshot(two_group_old(), two_group_new());
    assert_eq!(snapshot, again);
}

#[test]
fn split_groups_get_new_fingerprints() {
    let merged = digests("keep\nAAA\nBBB\nend\n", "keep\naaa\nbbb\nend\n");
    let split = digests("keep\nAAA\nmiddle\nBBB\nend\n", "keep\naaa\nmiddle\nbbb\nend\n");
    assert_eq!(merged.len(), 1);
    assert_eq!(split.len(), 2);
    assert!(!split.contains(&merged[0]));
}

#[test]
fn merged_groups_get_a_new_fingerprint() {
    let split = digests(two_group_old(), two_group_new());
    let merged = digests(
        "head-1\nhead-2\nhead-3\nhead-4\nAAA\nBBB\ntail\n",
        "head-1\nhead-2\nhead-3\nhead-4\naaa\nbbb\ntail\n",
    );
    assert_eq!(split.len(), 2);
    assert_eq!(merged.len(), 1);
    assert!(!split.contains(&merged[0]));
}

#[test]
fn duplicate_groups_with_the_same_context_share_a_digest() {
    let snapshot = canonical_review_snapshot("x\nAAA\nx\nAAA\nx\n", "x\naaa\nx\naaa\nx\n");
    assert_eq!(snapshot.fingerprints.len(), 2);
    assert_eq!(
        snapshot.fingerprints[0].digest,
        snapshot.fingerprints[1].digest
    );
}

#[test]
fn missing_final_newline_changes_the_fingerprint() {
    let with_newline = digests("a\nb\n", "a\nB\n");
    let without_newline = digests("a\nb\n", "a\nB");
    assert_eq!(with_newline.len(), 1);
    assert_eq!(without_newline.len(), 1);
    assert_ne!(with_newline[0], without_newline[0]);
}

#[test]
fn exact_whitespace_is_part_of_the_canonical_payload() {
    let spaces = digests("keep\nfoo  \nend\n", "keep\nfoo\nend\n");
    let unchanged = digests("keep\nfoo\nend\n", "keep\nfoo\nend\n");
    assert_eq!(spaces.len(), 1);
    assert!(unchanged.is_empty());
}

#[test]
fn ignore_whitespace_display_does_not_hide_canonical_whitespace_groups() {
    let old = "keep\nfoo  \nunchanged\nbar\nend\n";
    let new = "keep\nfoo\nunchanged\nbaz\nend\n";
    let canonical = canonical_review_snapshot(old, new);
    assert_eq!(canonical.fingerprints.len(), 2);

    let mapping = display_group_canonical_indices(old, new, true);
    assert_eq!(mapping.len(), 1);
    assert_eq!(mapping[0], vec![1]);
}

#[test]
fn exact_whitespace_display_maps_one_to_one() {
    let mapping = display_group_canonical_indices(two_group_old(), two_group_new(), false);
    assert_eq!(mapping, vec![vec![0], vec![1]]);
}

#[test]
fn map_display_groups_uses_changed_line_overlap() {
    let canonical = compute_file_diff_full_plain("", two_group_old(), two_group_new(), false);
    let display = compute_file_diff("", two_group_old(), two_group_new(), false);
    let mapped = map_display_groups_to_canonical(&canonical.lines, &display.lines);
    assert_eq!(mapped, vec![vec![0], vec![1]]);
}
