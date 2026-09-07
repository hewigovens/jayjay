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
    assert_eq!(
        digests("keep\r\n\r\nA\r\nend", "keep\r\n\r\na\r\nend"),
        vec!["498cda66f9d6a31c4e49769bf3ab9c5626f3140d41ffc78a973217bc02c17f62"]
    );
}

#[test]
fn fingerprints_depend_only_on_payload_and_adjacent_context() {
    let before = digests(two_group_old(), two_group_new());
    assert_eq!(before.len(), 2);

    let edited = digests(
        two_group_old(),
        "head-1\nhead-2\nhead-3\nhead-4\naaa-edited\nmiddle\nbbb\ntail\n",
    );
    assert_ne!(before[0], edited[0]);
    assert_eq!(before[1], edited[1]);

    let shifted = digests(
        "inserted\nhead-1\nhead-2\nhead-3\nhead-4\nAAA\nmiddle\nBBB\ntail\n",
        "inserted\nhead-1\nhead-2\nhead-3\nhead-4\naaa\nmiddle\nbbb\ntail\n",
    );
    assert_eq!(before, shifted);

    let alone = digests("a\nb\nAAA\nc\nd\ne\n", "a\nb\naaa\nc\nd\ne\n");
    let with_nearby_hunk = digests("a\nb\nAAA\nc\nd\ne\n", "a\nb\naaa\nc\nNEW\nd\ne\n");
    assert_eq!(with_nearby_hunk.len(), 2);
    assert_eq!(alone[0], with_nearby_hunk[0]);

    let original = digests("ctx-a\nAAA\nctx-b\n", "ctx-a\naaa\nctx-b\n");
    assert_ne!(
        original,
        digests("ctx-x\nAAA\nctx-b\n", "ctx-x\naaa\nctx-b\n")
    );
    assert_ne!(
        original,
        digests("ctx-a\nAAA\nctx-y\n", "ctx-a\naaa\nctx-y\n")
    );

    let duplicates = digests(
        "x\nx\nx\nAAA\nx\nx\nx\nAAA\nx\nx\nx\n",
        "x\nx\nx\naaa\nx\nx\nx\naaa\nx\nx\nx\n",
    );
    assert_eq!(duplicates.len(), 2);
    assert_eq!(duplicates[0], duplicates[1]);
}

#[test]
fn ignore_whitespace_display_does_not_hide_canonical_whitespace_groups() {
    let old = "keep\nfoo  \nunchanged\nbar\nend\n";
    let new = "keep\nfoo\nunchanged\nbaz\nend\n";
    assert_eq!(canonical_review_snapshot(old, new).fingerprints.len(), 2);
    assert_eq!(
        display_group_canonical_indices(old, new, false),
        vec![vec![0], vec![1]]
    );
    assert_eq!(
        display_group_canonical_indices(old, new, true),
        vec![vec![1]]
    );
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
