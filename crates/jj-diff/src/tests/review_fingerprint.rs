use crate::review_fingerprint::{canonical_review_snapshot, display_group_canonical_indices};

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
fn fingerprint_version_one_payload_is_stable() {
    assert_eq!(
        digests("keep\nA\rB\nend\n", "keep\na\rb\nend\n"),
        vec!["70bde24f42d428a41290a67366e47d697938453255cfdb9da9ce69a04f932305"]
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
fn inserting_a_hunk_within_the_context_window_preserves_the_neighbor() {
    let before = digests("a\nb\nAAA\nc\nd\ne\n", "a\nb\naaa\nc\nd\ne\n");
    let after = digests("a\nb\nAAA\nc\nd\ne\n", "a\nb\naaa\nc\nNEW\nd\ne\n");
    assert_eq!(after.len(), 2);
    assert_eq!(before[0], after[0]);
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
fn duplicate_groups_with_the_same_context_share_a_digest() {
    let snapshot = canonical_review_snapshot(
        "x\nx\nx\nAAA\nx\nx\nx\nAAA\nx\nx\nx\n",
        "x\nx\nx\naaa\nx\nx\nx\naaa\nx\nx\nx\n",
    );
    assert_eq!(snapshot.fingerprints.len(), 2);
    assert_eq!(
        snapshot.fingerprints[0].digest,
        snapshot.fingerprints[1].digest
    );
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
fn payload_and_context_are_exact_bytes() {
    let lf = digests("keep\nAAA\nend\n", "keep\naaa\nend\n");
    assert_ne!(
        digests("keep\r\nAAA\r\nend\r\n", "keep\r\naaa\r\nend\r\n"),
        lf
    );
    assert_ne!(digests("keep\r\nAAA\nend\n", "keep\r\naaa\nend\n"), lf);
    assert_ne!(digests("a\nb\n", "a\nB"), digests("a\nb\n", "a\nB\n"));
    assert_eq!(digests("keep\nfoo  \nend\n", "keep\nfoo\nend\n").len(), 1);
}
