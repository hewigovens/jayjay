use jayjay_primitives::{ReviewFileRollup, ReviewFileState, ReviewGroupState};
use jj_diff::{ReviewFileSnapshot, canonical_review_snapshot, display_group_canonical_indices};

use super::file_state::display_group_states;
use super::store::{ReviewEntry, key};
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

fn current(store: &ReviewStore, identity: &str, snapshot: &ReviewFileSnapshot) -> ReviewFileState {
    store.file_state(
        "c1",
        "a.txt",
        identity,
        Some(snapshot.fingerprints.as_slice()),
    )
}

fn states(
    store: &ReviewStore,
    identity: &str,
    snapshot: &ReviewFileSnapshot,
) -> Vec<ReviewGroupState> {
    current(store, identity, snapshot).group_states().to_vec()
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
fn moving_an_identical_patch_to_different_context_becomes_changed() {
    let mut store = store();
    let before = snap("ctx-a\nAAA\nctx-b\n", "ctx-a\naaa\nctx-b\n");
    mark_file(&mut store, "id-v1", &before);

    let after = snap("ctx-x\nAAA\nctx-y\n", "ctx-x\naaa\nctx-y\n");
    assert_eq!(
        states(&store, "id-v2", &after),
        vec![ReviewGroupState::ChangedSinceReview]
    );
    assert_eq!(current(&store, "id-v2", &after).removed_reviewed_count, 1);
}

#[test]
fn duplicate_ambiguous_fingerprints_do_not_inherit_reviewed_state() {
    let mut store = store();
    let unique = snap("x\nAAA\ny\n", "x\naaa\ny\n");
    mark_file(&mut store, "id-v1", &unique);

    let duplicates = snap(
        "x\nx\nx\nAAA\nx\nx\nx\nAAA\nx\nx\nx\n",
        "x\nx\nx\naaa\nx\nx\nx\naaa\nx\nx\nx\n",
    );
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
    let state = current(&store, "id-v2", &after);
    assert_eq!(state.group_states(), vec![ReviewGroupState::Reviewed]);
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
    let state = current(&store, "id-v2", &after);
    assert_eq!(state.group_states(), vec![ReviewGroupState::Reviewed]);
    assert_eq!(state.removed_reviewed_count, 0);
    assert_eq!(state.rollup(), ReviewFileRollup::Reviewed);
}

#[test]
fn whitespace_hidden_groups_keep_canonical_changed_rollup() {
    let mut store = store();
    let old = "keep\nfoo  \nunchanged\nbar\nend\n";
    let new = "keep\nfoo\nunchanged\nbaz\nend\n";
    let snapshot = snap(old, new);
    mark_file(&mut store, "id-v1", &snapshot);

    let edited = snap(old, "keep\nfoo\nunchanged\nbaz-edited\nend\n");
    let canonical = current(&store, "id-v2", &edited);
    assert_eq!(
        canonical.group_states(),
        vec![
            ReviewGroupState::Reviewed,
            ReviewGroupState::ChangedSinceReview
        ]
    );

    let mapping =
        display_group_canonical_indices(old, "keep\nfoo\nunchanged\nbaz-edited\nend\n", true);
    let display = display_group_states(&canonical, &mapping);
    assert_eq!(display, vec![ReviewGroupState::ChangedSinceReview]);
    assert_eq!(canonical.rollup(), ReviewFileRollup::ChangedSinceReview);
}

#[test]
fn no_review_entry_leaves_current_groups_unreviewed() {
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
fn snapshotless_matching_identity_applies_file_and_hunk_marks() {
    let mut store = store();
    store
        .state
        .reviewed
        .insert(key("c1", "a.txt"), ReviewEntry::hunks("id-v1", vec![1]));
    let snapshot = snap(two_group_old(), two_group_new());
    assert_eq!(
        states(&store, "id-v1", &snapshot),
        vec![ReviewGroupState::Unreviewed, ReviewGroupState::Reviewed]
    );

    store
        .state
        .reviewed
        .insert(key("c1", "a.txt"), ReviewEntry::file("id-v1"));
    assert_eq!(
        states(&store, "id-v1", &snapshot),
        vec![ReviewGroupState::Reviewed, ReviewGroupState::Reviewed]
    );
}

#[test]
fn snapshotless_hunks_with_mismatched_identity_do_not_guess() {
    let mut store = store();
    store
        .state
        .reviewed
        .insert(key("c1", "a.txt"), ReviewEntry::hunks("id-v1", vec![0]));
    let snapshot = snap(two_group_old(), two_group_new());
    assert_eq!(
        states(&store, "id-v2", &snapshot),
        vec![ReviewGroupState::Unreviewed, ReviewGroupState::Unreviewed]
    );
    assert_eq!(
        current(&store, "id-v2", &snapshot).removed_reviewed_count,
        0
    );
}

#[cfg(feature = "storage")]
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
        current(&reloaded, "id-a", &first).group_states(),
        vec![ReviewGroupState::Reviewed, ReviewGroupState::Reviewed]
    );
    assert!(reloaded.is_reviewed("c1", "b.txt", "id-b"));
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
fn reviewing_every_current_group_clears_removed_tombstones() {
    let mut store = store();
    let before = snap(separated_two_group_old(), separated_two_group_new());
    mark_file(&mut store, "id-v1", &before);
    let after = snap(
        "head-1\nhead-2\nhead-3\nhead-4\nAAA\nmid-1\nmid-2\nmid-3\ntail\n",
        "head-1\nhead-2\nhead-3\nhead-4\naaa\nmid-1\nmid-2\nmid-3\ntail\n",
    );
    assert_eq!(
        current(&store, "id-v2", &after).rollup(),
        ReviewFileRollup::ChangedSinceReview
    );

    store.mark_hunk_reviewed_snapshot("c1", "a.txt", "id-v2", Some(&after), 0);

    let state = current(&store, "id-v2", &after);
    assert!(state.is_fully_reviewed());
    assert_eq!(state.removed_reviewed_count, 0);
}

#[test]
fn rollup_uses_current_snapshot_after_identity_only_shift() {
    let mut store = store();
    let snapshot = snap(two_group_old(), two_group_new());
    mark_file(&mut store, "id-v1", &snapshot);
    assert_eq!(
        store.file_rollup("c1", "a.txt", "id-v2", None),
        ReviewFileRollup::ChangedSinceReview
    );
    assert_eq!(
        store.file_rollup("c1", "a.txt", "id-v2", Some(&snapshot)),
        ReviewFileRollup::Reviewed
    );
}

#[test]
fn identity_only_file_mark_is_reviewed_until_the_file_changes() {
    let mut store = store();
    store.mark_reviewed("c1", "a.txt", "id-v1");
    assert!(store.is_reviewed("c1", "a.txt", "id-v1"));
    assert_eq!(
        store.file_rollup("c1", "a.txt", "id-v2", None),
        ReviewFileRollup::ChangedSinceReview
    );
    let snapshot = snap(two_group_old(), two_group_new());
    let changed = current(&store, "id-v2", &snapshot);
    assert_eq!(
        changed.group_states(),
        vec![
            ReviewGroupState::ChangedSinceReview,
            ReviewGroupState::ChangedSinceReview
        ]
    );
    assert_eq!(changed.removed_reviewed_count, 0);
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

#[cfg(feature = "storage")]
#[test]
fn load_discards_old_baseline_map() {
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
    assert!(!loaded.state.extra.contains_key("review_baselines"));
}
