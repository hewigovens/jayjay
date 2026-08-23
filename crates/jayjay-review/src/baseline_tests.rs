use jayjay_primitives::{ReviewFileRollup, ReviewGroupState};
use jj_diff::{canonical_review_snapshot, display_group_canonical_indices, ReviewFileSnapshot};

use super::file_state::{display_group_states, mirror_digest};
use super::store::{ReviewEntry, StoredReviews, key};
use super::*;

fn store() -> ReviewStore {
    ReviewStore::in_memory()
}

fn two_group_old() -> &'static str {
    "head-1\nhead-2\nhead-3\nhead-4\nAAA\nmiddle\nBBB\ntail\n"
}

fn two_group_new() -> &'static str {
    "head-1\nhead-2\nhead-3\nhead-4\naaa\nmiddle\nbbb\ntail\n"
}

fn separated_two_group_old() -> &'static str {
    "head-1\nhead-2\nhead-3\nhead-4\nAAA\nmid-1\nmid-2\nmid-3\nBBB\ntail\n"
}

fn separated_two_group_new() -> &'static str {
    "head-1\nhead-2\nhead-3\nhead-4\naaa\nmid-1\nmid-2\nmid-3\nbbb\ntail\n"
}

fn snap(old: &str, new: &str) -> ReviewFileSnapshot {
    canonical_review_snapshot(old, new)
}

fn mark_file(store: &mut ReviewStore, identity: &str, snapshot: &ReviewFileSnapshot) {
    store.mark_reviewed_snapshot("c1", "a.txt", identity, Some(snapshot));
}

fn mark_group(store: &mut ReviewStore, identity: &str, snapshot: &ReviewFileSnapshot, index: u32) {
    store.mark_hunk_reviewed_snapshot("c1", "a.txt", identity, Some(snapshot), index);
}

fn states(
    store: &ReviewStore,
    identity: &str,
    snapshot: &ReviewFileSnapshot,
) -> Vec<ReviewGroupState> {
    store
        .file_state(
            "c1",
            "a.txt",
            identity,
            Some(snapshot.fingerprints.as_slice()),
        )
        .group_states
}

#[test]
fn editing_one_of_several_groups_invalidates_only_that_group() {
    let mut store = store();
    let before = snap(two_group_old(), two_group_new());
    mark_file(&mut store, "id-v1", &before);

    let after = snap(
        two_group_old(),
        "head-1\nhead-2\nhead-3\nhead-4\naaa-edited\nmiddle\nbbb\ntail\n",
    );
    assert_eq!(
        states(&store, "id-v2", &after),
        vec![
            ReviewGroupState::ChangedSinceReview,
            ReviewGroupState::Reviewed
        ]
    );
}

#[test]
fn inserting_lines_above_a_reviewed_group_preserves_it() {
    let mut store = store();
    let before = snap(two_group_old(), two_group_new());
    mark_file(&mut store, "id-v1", &before);

    let after = snap(
        "inserted\nhead-1\nhead-2\nhead-3\nhead-4\nAAA\nmiddle\nBBB\ntail\n",
        "inserted\nhead-1\nhead-2\nhead-3\nhead-4\naaa\nmiddle\nbbb\ntail\n",
    );
    assert_eq!(
        states(&store, "id-v2", &after),
        vec![ReviewGroupState::Reviewed, ReviewGroupState::Reviewed]
    );
}

#[test]
fn adding_a_distant_group_preserves_reviewed_existing_groups() {
    let mut store = store();
    let before = snap(two_group_old(), two_group_new());
    mark_file(&mut store, "id-v1", &before);

    let after = snap(
        "head-1\nhead-2\nhead-3\nhead-4\nAAA\nmiddle\nBBB\ntail\nzzz\n",
        "head-1\nhead-2\nhead-3\nhead-4\naaa\nmiddle\nbbb\ntail\nZZZ\n",
    );
    assert_eq!(
        states(&store, "id-v2", &after),
        vec![
            ReviewGroupState::Reviewed,
            ReviewGroupState::Reviewed,
            ReviewGroupState::ChangedSinceReview
        ]
    );
}

#[test]
fn moving_an_identical_patch_to_different_context_becomes_changed() {
    let mut store = store();
    let before = snap("ctx-a\nAAA\nctx-b\n", "ctx-a\naaa\nctx-b\n");
    mark_file(&mut store, "id-v1", &before);

    let after = snap("ctx-x\nAAA\nctx-y\n", "ctx-x\naaa\nctx-y\n");
    assert_eq!(
        states(&store, "id-v2", &after),
        vec![ReviewGroupState::ChangedSinceReview]
    );
    assert_eq!(
        store
            .file_state("c1", "a.txt", "id-v2", Some(after.fingerprints.as_slice()))
            .removed_reviewed_count,
        1
    );
}

#[test]
fn split_groups_become_changed() {
    let mut store = store();
    let merged = snap("keep\nAAA\nBBB\nend\n", "keep\naaa\nbbb\nend\n");
    mark_file(&mut store, "id-v1", &merged);

    let split = snap(
        "keep\nAAA\nmiddle\nBBB\nend\n",
        "keep\naaa\nmiddle\nbbb\nend\n",
    );
    assert_eq!(
        states(&store, "id-v2", &split),
        vec![
            ReviewGroupState::ChangedSinceReview,
            ReviewGroupState::ChangedSinceReview
        ]
    );
}

#[test]
fn merged_groups_become_changed() {
    let mut store = store();
    let split = snap(two_group_old(), two_group_new());
    mark_file(&mut store, "id-v1", &split);

    let merged = snap(
        "head-1\nhead-2\nhead-3\nhead-4\nAAA\nBBB\ntail\n",
        "head-1\nhead-2\nhead-3\nhead-4\naaa\nbbb\ntail\n",
    );
    assert_eq!(
        states(&store, "id-v2", &merged),
        vec![ReviewGroupState::ChangedSinceReview]
    );
}

#[test]
fn duplicate_ambiguous_fingerprints_do_not_inherit_reviewed_state() {
    let mut store = store();
    let unique = snap("x\nAAA\ny\n", "x\naaa\ny\n");
    mark_file(&mut store, "id-v1", &unique);

    let duplicates = snap("x\nAAA\nx\nAAA\nx\n", "x\naaa\nx\naaa\nx\n");
    assert_eq!(duplicates.fingerprints[0], duplicates.fingerprints[1]);
    assert_eq!(
        states(&store, "id-v2", &duplicates),
        vec![
            ReviewGroupState::ChangedSinceReview,
            ReviewGroupState::ChangedSinceReview
        ]
    );
}

#[test]
fn a_removed_reviewed_group_keeps_the_file_changed() {
    let mut store = store();
    let before = snap(separated_two_group_old(), separated_two_group_new());
    mark_file(&mut store, "id-v1", &before);

    let after = snap(
        "head-1\nhead-2\nhead-3\nhead-4\nAAA\nmid-1\nmid-2\nmid-3\ntail\n",
        "head-1\nhead-2\nhead-3\nhead-4\naaa\nmid-1\nmid-2\nmid-3\ntail\n",
    );
    let state = store.file_state("c1", "a.txt", "id-v2", Some(after.fingerprints.as_slice()));
    assert_eq!(state.group_states, vec![ReviewGroupState::Reviewed]);
    assert_eq!(state.removed_reviewed_count, 1);
    assert_eq!(state.rollup(), ReviewFileRollup::ChangedSinceReview);
    assert!(!state.is_fully_reviewed());
}

#[test]
fn a_removed_unreviewed_group_does_not_create_a_reviewed_removal_warning() {
    let mut store = store();
    let before = snap(separated_two_group_old(), separated_two_group_new());
    mark_group(&mut store, "id-v1", &before, 0);

    let after = snap(
        "head-1\nhead-2\nhead-3\nhead-4\nAAA\nmid-1\nmid-2\nmid-3\ntail\n",
        "head-1\nhead-2\nhead-3\nhead-4\naaa\nmid-1\nmid-2\nmid-3\ntail\n",
    );
    let state = store.file_state("c1", "a.txt", "id-v2", Some(after.fingerprints.as_slice()));
    assert_eq!(state.group_states, vec![ReviewGroupState::Reviewed]);
    assert_eq!(state.removed_reviewed_count, 0);
    assert_eq!(state.rollup(), ReviewFileRollup::Reviewed);
}

#[test]
fn crlf_to_lf_invalidates_a_reviewed_group() {
    let mut store = store();
    let crlf = snap("keep\r\nAAA\r\nend\r\n", "keep\r\naaa\r\nend\r\n");
    mark_file(&mut store, "id-v1", &crlf);
    let lf = snap("keep\nAAA\nend\n", "keep\naaa\nend\n");
    assert_eq!(
        states(&store, "id-v2", &lf),
        vec![ReviewGroupState::ChangedSinceReview]
    );
}

#[test]
fn missing_final_newline_changes_are_detected() {
    let mut store = store();
    let with_newline = snap("a\nb\n", "a\nB\n");
    mark_file(&mut store, "id-v1", &with_newline);

    let without_newline = snap("a\nb\n", "a\nB");
    assert_eq!(
        states(&store, "id-v2", &without_newline),
        vec![ReviewGroupState::ChangedSinceReview]
    );
}

#[test]
fn whitespace_hidden_groups_keep_canonical_changed_rollup() {
    let mut store = store();
    let old = "keep\nfoo  \nunchanged\nbar\nend\n";
    let new = "keep\nfoo\nunchanged\nbaz\nend\n";
    let snapshot = snap(old, new);
    mark_file(&mut store, "id-v1", &snapshot);

    let edited = snap(old, "keep\nfoo\nunchanged\nbaz-edited\nend\n");
    let canonical = store.file_state(
        "c1",
        "a.txt",
        "id-v2",
        Some(edited.fingerprints.as_slice()),
    );
    assert_eq!(
        canonical.group_states,
        vec![
            ReviewGroupState::Reviewed,
            ReviewGroupState::ChangedSinceReview
        ]
    );

    let mapping = display_group_canonical_indices(old, "keep\nfoo\nunchanged\nbaz-edited\nend\n", true);
    let display = display_group_states(&canonical, &mapping);
    assert_eq!(display, vec![ReviewGroupState::ChangedSinceReview]);
    assert_eq!(canonical.rollup(), ReviewFileRollup::ChangedSinceReview);
}

#[test]
fn no_baseline_current_groups_are_unreviewed() {
    let store = store();
    let snapshot = snap(two_group_old(), two_group_new());
    assert_eq!(
        states(&store, "id-v1", &snapshot),
        vec![ReviewGroupState::Unreviewed, ReviewGroupState::Unreviewed]
    );
}

#[test]
fn byte_identical_rebase_preserves_state() {
    let mut store = store();
    let snapshot = snap(two_group_old(), two_group_new());
    mark_group(&mut store, "id-v1", &snapshot, 1);

    assert_eq!(
        states(&store, "id-v1", &snapshot),
        vec![ReviewGroupState::Unreviewed, ReviewGroupState::Reviewed]
    );
}

#[test]
fn legacy_matching_identity_migrates_file_and_hunk_marks() {
    let mut store = store();
    store.state.reviewed.insert(
        key("c1", "a.txt"),
        ReviewEntry::marked_hunks("id-v1", vec![1]),
    );
    let snapshot = snap(two_group_old(), two_group_new());
    assert_eq!(
        states(&store, "id-v1", &snapshot),
        vec![ReviewGroupState::Unreviewed, ReviewGroupState::Reviewed]
    );

    store.state.reviewed.insert(
        key("c1", "a.txt"),
        ReviewEntry::marked_file("id-v1"),
    );
    assert_eq!(
        states(&store, "id-v1", &snapshot),
        vec![ReviewGroupState::Reviewed, ReviewGroupState::Reviewed]
    );
}

#[test]
fn legacy_mismatched_identity_does_not_guess() {
    let mut store = store();
    store.state.reviewed.insert(
        key("c1", "a.txt"),
        ReviewEntry::marked_file("id-v1"),
    );
    let snapshot = snap(two_group_old(), two_group_new());
    assert_eq!(
        states(&store, "id-v2", &snapshot),
        vec![ReviewGroupState::Unreviewed, ReviewGroupState::Unreviewed]
    );
    assert_eq!(
        store
            .file_state("c1", "a.txt", "id-v2", Some(snapshot.fingerprints.as_slice()))
            .removed_reviewed_count,
        0
    );
}

#[test]
fn json_round_trip_preserves_unknown_fields() {
    let json = r#"{
        "reviewed": {
            "c1|a.txt": {
                "identity": "id",
                "file_marked": true,
                "hunks": [],
                "entry_future": 1
            }
        },
        "review_baselines": {
            "c1|a.txt": {
                "schema_version": 1,
                "algorithm_version": 1,
                "identity": "id",
                "groups": [{"digest": "abc", "state": "reviewed", "group_future": true}],
                "mirror_digest": "stale",
                "baseline_future": {"ok": true}
            }
        },
        "top_future": true
    }"#;
    let parsed: StoredReviews = serde_json::from_str(json).unwrap();
    let mut store = ReviewStore::from_state(parsed, true);
    store.mark_reviewed("c2", "b.txt", "id-b");

    let text = serde_json::to_string(&store.state).unwrap();
    assert!(text.contains(r#""entry_future":1"#), "{text}");
    assert!(text.contains(r#""group_future":true"#), "{text}");
    assert!(text.contains(r#""baseline_future":{"ok":true}"#), "{text}");
    assert!(text.contains(r#""top_future":true"#), "{text}");
}

#[test]
fn stale_mirror_digest_falls_back_to_legacy_instead_of_guessing_fingerprints() {
    let snapshot = snap(two_group_old(), two_group_new());
    let mut store = store();
    mark_file(&mut store, "id-v1", &snapshot);

    let k = key("c1", "a.txt");
    store.state.reviewed.get_mut(&k).unwrap().hunks = vec![0];
    store.state.reviewed.get_mut(&k).unwrap().file_marked = false;
    assert_ne!(
        store.state.review_baselines[&k].mirror_digest,
        mirror_digest(&store.state.reviewed[&k])
    );

    assert_eq!(
        states(&store, "id-v1", &snapshot),
        vec![ReviewGroupState::Reviewed, ReviewGroupState::Unreviewed]
    );
}

#[test]
fn two_independently_loaded_writers_preserve_unrelated_review_state() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("review_store.json");
    let first = snap(two_group_old(), two_group_new());
    let second = snap("z\nAAA\n", "z\naaa\n");

    let mut writer_a = ReviewStore::load_from(path.clone());
    writer_a.mark_reviewed_snapshot("c1", "a.txt", "id-a", Some(&first));

    let mut writer_b = ReviewStore::load_from(path.clone());
    writer_b.mark_reviewed_snapshot("c1", "b.txt", "id-b", Some(&second));

    writer_a.refresh_from_disk();
    writer_a.mark_hunk_reviewed_snapshot("c1", "a.txt", "id-a", Some(&first), 0);

    let reloaded = ReviewStore::load_from(path);
    assert_eq!(
        reloaded
            .file_state("c1", "a.txt", "id-a", Some(first.fingerprints.as_slice()))
            .group_states,
        vec![ReviewGroupState::Reviewed, ReviewGroupState::Reviewed]
    );
    assert!(reloaded.is_reviewed("c1", "b.txt", "id-b"));
}

#[test]
fn clear_change_drops_baselines_only_for_that_change() {
    let mut store = store();
    let snapshot = snap(two_group_old(), two_group_new());
    store.mark_reviewed_snapshot("c1", "a.txt", "id-v1", Some(&snapshot));
    store.mark_reviewed_snapshot("c2", "a.txt", "id-v1", Some(&snapshot));
    store.clear_change("c1");

    assert!(!store.state.reviewed.contains_key(&key("c1", "a.txt")));
    assert!(!store.state.review_baselines.contains_key(&key("c1", "a.txt")));
    assert!(store.is_reviewed("c2", "a.txt", "id-v1"));
    assert!(store.state.review_baselines.contains_key(&key("c2", "a.txt")));
}

#[test]
fn marking_one_changed_group_does_not_mark_siblings() {
    let mut store = store();
    let before = snap(two_group_old(), two_group_new());
    mark_file(&mut store, "id-v1", &before);

    let after = snap(
        two_group_old(),
        "head-1\nhead-2\nhead-3\nhead-4\naaa-edited\nmiddle\nbbb\ntail\n",
    );
    store.mark_hunk_reviewed_snapshot("c1", "a.txt", "id-v2", Some(&after), 0);
    assert_eq!(
        states(&store, "id-v2", &after),
        vec![ReviewGroupState::Reviewed, ReviewGroupState::Reviewed]
    );

    let after_both_changed = snap(
        two_group_old(),
        "head-1\nhead-2\nhead-3\nhead-4\naaa-edited\nmiddle\nbbb-edited\ntail\n",
    );
    store.mark_reviewed_snapshot("c1", "a.txt", "id-v1", Some(&before));
    store.mark_hunk_reviewed_snapshot("c1", "a.txt", "id-v3", Some(&after_both_changed), 0);
    assert_eq!(
        states(&store, "id-v3", &after_both_changed),
        vec![
            ReviewGroupState::Reviewed,
            ReviewGroupState::ChangedSinceReview
        ]
    );
}

#[test]
fn duplicate_group_extras_are_copied_by_occurrence() {
    let mut store = store();
    let snapshot = snap("x\nAAA\nx\nAAA\nx\n", "x\naaa\nx\naaa\nx\n");
    mark_file(&mut store, "id-v1", &snapshot);
    let k = key("c1", "a.txt");
    {
        let groups = &mut store.state.review_baselines.get_mut(&k).unwrap().groups;
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].digest, groups[1].digest);
        groups[0]
            .extra
            .insert("group_future".into(), serde_json::json!(1));
        groups[1]
            .extra
            .insert("group_future".into(), serde_json::json!(2));
    }
    store.mark_hunk_reviewed_snapshot("c1", "a.txt", "id-v1", Some(&snapshot), 0);
    let groups = &store.state.review_baselines[&k].groups;
    assert_eq!(
        groups[0].extra.get("group_future"),
        Some(&serde_json::json!(1))
    );
    assert_eq!(
        groups[1].extra.get("group_future"),
        Some(&serde_json::json!(2))
    );
}

#[test]
fn stale_mirror_does_not_seed_tombstones_on_later_hunk_mutation() {
    let mut store = store();
    let before = snap(separated_two_group_old(), separated_two_group_new());
    mark_file(&mut store, "id-v1", &before);
    let k = key("c1", "a.txt");
    store.state.reviewed.get_mut(&k).unwrap().file_marked = false;
    store.state.reviewed.get_mut(&k).unwrap().hunks = vec![0];
    assert_ne!(
        store.state.review_baselines[&k].mirror_digest,
        mirror_digest(&store.state.reviewed[&k])
    );

    let after = snap(
        "head-1\nhead-2\nhead-3\nhead-4\nAAA\nmid-1\nmid-2\nmid-3\ntail\n",
        "head-1\nhead-2\nhead-3\nhead-4\naaa\nmid-1\nmid-2\nmid-3\ntail\n",
    );
    store.mark_hunk_reviewed_snapshot("c1", "a.txt", "id-v2", Some(&after), 0);
    assert!(
        store.state.review_baselines[&k]
            .removed_reviewed
            .is_empty(),
        "{:?}",
        store.state.review_baselines[&k].removed_reviewed
    );
    assert_eq!(
        states(&store, "id-v2", &after),
        vec![ReviewGroupState::Reviewed]
    );
}

#[test]
fn hunk_mutation_records_removed_reviewed_from_a_trusted_baseline() {
    let mut store = store();
    let before = snap(separated_two_group_old(), separated_two_group_new());
    mark_file(&mut store, "id-v1", &before);
    let after = snap(
        "head-1\nhead-2\nhead-3\nhead-4\nAAA\nmid-1\nmid-2\nmid-3\ntail\n",
        "head-1\nhead-2\nhead-3\nhead-4\naaa\nmid-1\nmid-2\nmid-3\ntail\n",
    );
    store.mark_hunk_reviewed_snapshot("c1", "a.txt", "id-v2", Some(&after), 0);
    assert_eq!(
        store.state.review_baselines[&key("c1", "a.txt")]
            .removed_reviewed
            .len(),
        1
    );
    assert_eq!(
        store
            .file_state("c1", "a.txt", "id-v2", Some(after.fingerprints.as_slice()))
            .rollup(),
        ReviewFileRollup::ChangedSinceReview
    );
}

#[test]
fn rollup_uses_current_snapshot_after_identity_only_shift() {
    let mut store = store();
    let snapshot = snap(two_group_old(), two_group_new());
    mark_file(&mut store, "id-v1", &snapshot);
    assert_eq!(
        store.file_rollup("c1", "a.txt", "id-v2"),
        ReviewFileRollup::ChangedSinceReview
    );
    assert_eq!(
        store.file_rollup_with_snapshot("c1", "a.txt", "id-v2", &snapshot),
        ReviewFileRollup::Reviewed
    );
}

#[test]
fn identity_only_file_mark_is_reviewed_until_the_file_changes() {
    let mut store = store();
    store.mark_reviewed("c1", "a.txt", "id-v1");
    assert!(store.is_reviewed("c1", "a.txt", "id-v1"));
    assert_eq!(
        store.file_rollup("c1", "a.txt", "id-v2"),
        ReviewFileRollup::ChangedSinceReview
    );
}

#[test]
fn unmarking_one_group_preserves_sibling_reviewed_state() {
    let mut store = store();
    let snapshot = snap(two_group_old(), two_group_new());
    mark_file(&mut store, "id-v1", &snapshot);
    store.mark_hunk_unreviewed_snapshot("c1", "a.txt", "id-v1", &snapshot, 0);
    assert_eq!(
        states(&store, "id-v1", &snapshot),
        vec![ReviewGroupState::Unreviewed, ReviewGroupState::Reviewed]
    );
}

#[test]
fn load_prunes_orphan_baselines_left_by_an_older_unreview() {
    let json = r#"{
        "reviewed": {},
        "review_baselines": {
            "c1|a.txt": {
                "schema_version": 1,
                "algorithm_version": 1,
                "identity": "id",
                "groups": [{"digest": "abc", "state": "reviewed"}]
            }
        }
    }"#;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("review_store.json");
    std::fs::write(&path, json).unwrap();
    let loaded = ReviewStore::load_from(path);
    assert!(loaded.state.review_baselines.is_empty());
}
