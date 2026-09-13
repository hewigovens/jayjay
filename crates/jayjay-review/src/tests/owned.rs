use super::*;

fn snapshot(digests: &[&str]) -> jj_diff::ReviewFileSnapshot {
    jj_diff::ReviewFileSnapshot {
        algorithm_version: jj_diff::REVIEW_FINGERPRINT_VERSION,
        fingerprints: digests
            .iter()
            .map(|digest| jj_diff::ReviewGroupFingerprint {
                digest: digest.to_string(),
            })
            .collect(),
    }
}

#[test]
fn agent_unmark_preserves_human_groups_and_files() {
    let mut s = make_store();
    let snapshot = snapshot(&["a", "b"]);

    s.mark_reviewed_as(
        "c1",
        "a.txt",
        "id-v1",
        Some(&snapshot),
        ReviewMarkSource::Agent,
    )
    .unwrap();
    s.mark_groups_reviewed_as(
        "c1",
        "b.txt",
        "id-v1",
        &snapshot,
        &[0],
        ReviewMarkSource::Agent,
    )
    .unwrap();
    s.mark_reviewed("c1", "c.txt", "id-v1");
    assert_eq!(
        s.paths_owned_by("c1", ReviewMarkSource::Agent),
        vec!["a.txt", "b.txt"]
    );
    assert!(s.is_reviewed("c1", "a.txt", "id-v1"));
    assert!(s.is_hunk_reviewed("c1", "b.txt", "id-v1", 0));
    assert!(!s.is_hunk_reviewed("c1", "b.txt", "id-v1", 1));
    assert!(s.snapshot_json().unwrap().contains("\"source\":\"agent\""));

    s.mark_hunk_reviewed_snapshot("c1", "b.txt", "id-v1", Some(&snapshot), 1);
    assert_eq!(
        s.paths_owned_by("c1", ReviewMarkSource::Agent),
        vec!["a.txt", "b.txt"]
    );

    assert_eq!(
        s.unmark_owned_by("c1", Some("c.txt"), ReviewMarkSource::Agent)
            .unwrap(),
        Vec::<String>::new()
    );
    assert!(s.is_reviewed("c1", "c.txt", "id-v1"));
    assert_eq!(
        s.unmark_owned_by("c1", None, ReviewMarkSource::Agent)
            .unwrap(),
        vec!["a.txt", "b.txt"]
    );
    assert!(!s.is_reviewed("c1", "a.txt", "id-v1"));
    assert!(s.is_hunk_reviewed("c1", "b.txt", "id-v1", 1));
    assert!(!s.is_hunk_reviewed("c1", "b.txt", "id-v1", 0));
}

#[test]
fn human_and_agent_groups_keep_separate_ownership() {
    let snapshot = snapshot(&["human", "agent"]);
    for human_first in [true, false] {
        let mut store = make_store();
        if human_first {
            store.mark_hunk_reviewed_snapshot("c", "a", "id", Some(&snapshot), 0);
        }
        store
            .mark_groups_reviewed_as("c", "a", "id", &snapshot, &[1], ReviewMarkSource::Agent)
            .unwrap();
        if !human_first {
            store.mark_hunk_reviewed_snapshot("c", "a", "id", Some(&snapshot), 0);
        }
        assert_eq!(store.paths_owned_by("c", ReviewMarkSource::Agent), ["a"]);
        let mut store = ReviewStore::in_memory_from_json(&store.snapshot_json().unwrap()).unwrap();
        store
            .unmark_owned_by("c", None, ReviewMarkSource::Agent)
            .unwrap();
        let state = store.file_review_state("c", "a", "id", Some(&snapshot));
        assert_eq!(
            state.group_states(),
            [ReviewGroupState::Reviewed, ReviewGroupState::Unreviewed]
        );
    }
}

#[test]
fn agent_whole_file_mark_preserves_human_groups_and_human_can_take_ownership() {
    let snapshot = snapshot(&["a", "b"]);
    let mut store = make_store();
    store.mark_hunk_reviewed_snapshot("c", "a", "id", Some(&snapshot), 0);
    store
        .mark_reviewed_as("c", "a", "id", Some(&snapshot), ReviewMarkSource::Agent)
        .unwrap();
    store
        .unmark_owned_by("c", None, ReviewMarkSource::Agent)
        .unwrap();
    assert_eq!(
        store
            .file_review_state("c", "a", "id", Some(&snapshot))
            .group_states(),
        [ReviewGroupState::Reviewed, ReviewGroupState::Unreviewed]
    );
    store
        .mark_reviewed_as("c", "a", "id", Some(&snapshot), ReviewMarkSource::Agent)
        .unwrap();
    store.mark_hunk_reviewed("c", "a", "id", 1);
    assert!(
        store
            .paths_owned_by("c", ReviewMarkSource::Agent)
            .is_empty()
    );
    assert!(store.is_reviewed("c", "a", "id"));
}

#[test]
fn group_ownership_survives_reordering_removal_and_reappearance() {
    let original = snapshot(&["human", "agent", "other"]);
    let removed = snapshot(&["human", "other"]);
    for reappears in [false, true] {
        let mut store = make_store();
        store.mark_hunk_reviewed_snapshot("c", "a", "v1", Some(&original), 0);
        store
            .mark_groups_reviewed_as("c", "a", "v1", &original, &[1], ReviewMarkSource::Agent)
            .unwrap();
        store.mark_hunk_reviewed_snapshot("c", "a", "v2", Some(&removed), 0);
        assert_eq!(store.paths_owned_by("c", ReviewMarkSource::Agent), ["a"]);
        let current = if reappears {
            snapshot(&["agent", "other", "human"])
        } else {
            removed.clone()
        };
        let human_index = if reappears { 2 } else { 0 };
        store.mark_hunk_reviewed_snapshot("c", "a", "v3", Some(&current), human_index);
        let mut store = ReviewStore::in_memory_from_json(&store.snapshot_json().unwrap()).unwrap();
        store
            .unmark_owned_by("c", None, ReviewMarkSource::Agent)
            .unwrap();
        let state = store.file_review_state("c", "a", "v3", Some(&current));
        assert_eq!(state.reviewed_indices(), [human_index]);
        assert_eq!(state.removed_reviewed_count, 0);
    }
}

#[test]
fn duplicate_groups_keep_human_ownership_and_removed_warning() {
    let original = snapshot(&["duplicate", "duplicate", "other"]);
    let mut store = make_store();
    store.mark_hunk_reviewed_snapshot("c", "a", "v1", Some(&original), 0);
    store
        .mark_groups_reviewed_as("c", "a", "v1", &original, &[1], ReviewMarkSource::Agent)
        .unwrap();
    let mut same = ReviewStore::in_memory_from_json(&store.snapshot_json().unwrap()).unwrap();
    same.unmark_owned_by("c", None, ReviewMarkSource::Agent)
        .unwrap();
    assert_eq!(
        same.file_review_state("c", "a", "v1", Some(&original))
            .reviewed_indices(),
        [0]
    );

    let removed = snapshot(&["other", "new"]);
    store.mark_hunk_reviewed_snapshot("c", "a", "v2", Some(&removed), 0);
    store
        .unmark_owned_by("c", None, ReviewMarkSource::Agent)
        .unwrap();
    let state = store.file_review_state("c", "a", "v2", Some(&removed));
    assert_eq!(state.reviewed_indices(), [0]);
    assert_eq!(state.removed_reviewed_count, 1);
}

#[test]
fn snapshot_less_batch_set_drops_an_agent_file_mark_instead_of_relabeling_it() {
    let mut store = make_store();
    store
        .mark_reviewed_as("c", "a", "id", None, ReviewMarkSource::Agent)
        .unwrap();
    store.set_reviewed_hunks("c", "a", "id", vec![1]);
    assert!(
        store
            .paths_owned_by("c", ReviewMarkSource::Agent)
            .is_empty()
    );
    assert!(!store.is_hunk_reviewed("c", "a", "id", 1));

    store.mark_reviewed("c", "a", "id");
    store.set_reviewed_hunks("c", "a", "id", vec![1]);
    assert!(store.is_hunk_reviewed("c", "a", "id", 1));
}

#[test]
fn unmarking_a_person_keeps_the_agent_groups() {
    let snapshot = snapshot(&["human", "agent"]);
    let mut store = make_store();
    store.mark_hunk_reviewed_snapshot("c", "a", "id", Some(&snapshot), 0);
    store
        .mark_groups_reviewed_as("c", "a", "id", &snapshot, &[1], ReviewMarkSource::Agent)
        .unwrap();
    assert_eq!(store.paths_owned_by("c", ReviewMarkSource::User), ["a"]);
    store
        .unmark_owned_by("c", None, ReviewMarkSource::User)
        .unwrap();
    assert!(store.paths_owned_by("c", ReviewMarkSource::User).is_empty());
    assert_eq!(store.paths_owned_by("c", ReviewMarkSource::Agent), ["a"]);
    assert_eq!(
        store
            .file_review_state("c", "a", "id", Some(&snapshot))
            .group_states(),
        [ReviewGroupState::Unreviewed, ReviewGroupState::Reviewed]
    );
    store
        .mark_reviewed_as("c", "a", "id", Some(&snapshot), ReviewMarkSource::User)
        .unwrap();
    assert!(
        store
            .paths_owned_by("c", ReviewMarkSource::Agent)
            .is_empty()
    );
}

#[test]
fn an_older_app_save_keeps_agent_ownership_at_the_entry_level() {
    let json = r#"{"reviewed":{"c|a":{"identity":"id","source":"agent","state":{"kind":"groups","algorithm_version":1,"groups":[{"digest":"a","state":"reviewed"},{"digest":"b","state":"unreviewed"}]}}}}"#;
    let mut store = ReviewStore::in_memory_from_json(json).unwrap();
    assert_eq!(store.paths_owned_by("c", ReviewMarkSource::Agent), ["a"]);
    assert!(store.paths_owned_by("c", ReviewMarkSource::User).is_empty());
    assert_eq!(
        store
            .unmark_owned_by("c", None, ReviewMarkSource::Agent)
            .unwrap(),
        ["a"]
    );
    assert!(
        store
            .paths_owned_by("c", ReviewMarkSource::Agent)
            .is_empty()
    );
    assert!(!store.is_hunk_reviewed("c", "a", "id", 0));
}

#[test]
fn entries_without_agent_marks_serialize_without_sources() {
    let snapshot = snapshot(&["a", "b"]);
    let mut store = make_store();
    store.mark_hunk_reviewed_snapshot("c", "a", "id", Some(&snapshot), 0);
    store
        .mark_reviewed_as("c", "b", "id", Some(&snapshot), ReviewMarkSource::User)
        .unwrap();
    assert!(!store.snapshot_json().unwrap().contains("source"));
}
