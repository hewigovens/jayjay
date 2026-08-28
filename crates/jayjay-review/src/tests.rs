use super::*;
use crate::store::{ReviewEntryState, StoredReviews};
use crate::test_util::SequentialIds;

#[cfg(feature = "storage")]
mod storage;

fn make_store() -> ReviewStore {
    ReviewStore::in_memory()
}

fn note_store() -> ReviewStore {
    ReviewStore::in_memory_with_ids(Box::new(SequentialIds::new()))
}

fn anchor() -> NoteAnchor {
    NoteAnchor {
        change_id: "c1".to_string(),
        path: "a.txt".to_string(),
        identity: "id-v1".to_string(),
        side: NoteSide::New,
        line: 2,
        anchor_excerpt: "new line".to_string(),
        anchor_context: vec!["new line".to_string()],
        ignore_whitespace: false,
    }
}

#[test]
fn file_mark_roundtrip() {
    let mut s = make_store();
    s.mark_reviewed("c1", "a.txt", "id-v1");
    assert!(s.is_reviewed("c1", "a.txt", "id-v1"));
}

#[test]
fn in_memory_snapshot_round_trips_review_state() {
    let mut store = note_store();
    store.mark_reviewed("c1", "a.txt", "id-v1");
    let note = store.add_note(anchor(), "check this");

    let snapshot = store.snapshot_json().unwrap();
    let restored = ReviewStore::in_memory_from_json(&snapshot).unwrap();

    assert!(restored.is_reviewed("c1", "a.txt", "id-v1"));
    assert_eq!(restored.list_notes("c1", false), vec![note]);
}

#[test]
fn identity_change_invalidates_marks() {
    let mut s = make_store();
    s.mark_reviewed("c1", "a.txt", "id-v1");
    s.mark_hunk_reviewed("c1", "a.txt", "id-v1", 0);
    assert!(!s.is_reviewed("c1", "a.txt", "id-v2"));
    assert!(!s.is_hunk_reviewed("c1", "a.txt", "id-v2", 0));
}

#[test]
fn matching_identity_keeps_marks() {
    // The store treats identity as opaque; rebase-invariance of the identity itself is proven against a real repo in tests/review_identity.rs.
    let mut s = make_store();
    s.mark_reviewed("c1", "a.txt", "id-v1");
    assert!(s.is_reviewed("c1", "a.txt", "id-v1"));
}

#[test]
fn empty_identity_is_a_no_op() {
    // Empty identity (e.g., file has no diff context) refuses to record.
    let mut s = make_store();
    s.mark_reviewed("c1", "a.txt", "");
    assert!(s.state.reviewed.is_empty());
}

#[test]
fn clear_change_only_drops_marks_for_that_change() {
    let mut s = make_store();
    s.mark_reviewed("c1", "a.txt", "id-v1");
    s.mark_reviewed("c2", "a.txt", "id-v1");
    s.clear_change("c1");
    assert!(!s.is_reviewed("c1", "a.txt", "id-v1"));
    assert!(s.is_reviewed("c2", "a.txt", "id-v1"));
}

#[test]
fn hunk_mark_independent_of_file_flag() {
    let mut s = make_store();
    s.mark_hunk_reviewed("c1", "a.txt", "id", 2);
    assert!(s.is_hunk_reviewed("c1", "a.txt", "id", 2));
    assert!(!s.is_hunk_reviewed("c1", "a.txt", "id", 0));
    assert!(!s.is_reviewed("c1", "a.txt", "id"));
}

#[test]
fn file_marked_rollup_and_demotion() {
    let mut s = make_store();
    s.mark_reviewed("c1", "a.txt", "id");
    assert!(s.is_hunk_reviewed("c1", "a.txt", "id", 999));
    s.mark_hunk_unreviewed("c1", "a.txt", 1);
    assert!(!s.is_reviewed("c1", "a.txt", "id"));
}

#[test]
fn json_load_migrates_pre_tag_entries_drops_junk_and_save_round_trips() {
    let json = r#"{"reviewed":{
        "c|file":{"identity":"id1","file_marked":true,"hunks":[1,3]},
        "c|hunks":{"identity":"id1","file_marked":false,"hunks":[3,1]},
        "c|empty":{"identity":"id1","file_marked":false},
        "c|junk":7,
        "c|new":{"identity":"id1","state":{"kind":"hunks","indices":[1,3]}}}}"#;
    let parsed: StoredReviews = serde_json::from_str(json).unwrap();
    assert!(matches!(
        parsed.reviewed["c|file"].state,
        ReviewEntryState::File
    ));
    assert!(matches!(
        &parsed.reviewed["c|hunks"].state,
        ReviewEntryState::Hunks { indices } if indices == &vec![1, 3]
    ));
    assert!(!parsed.reviewed.contains_key("c|empty"));
    assert!(!parsed.reviewed.contains_key("c|junk"));
    assert!(matches!(
        &parsed.reviewed["c|new"].state,
        ReviewEntryState::Hunks { indices } if indices == &vec![1, 3]
    ));
    assert!(parsed.notes.is_empty());

    let text = serde_json::to_string(&parsed).unwrap();
    let saved: StoredReviews = serde_json::from_str(&text).unwrap();
    assert_eq!(saved.reviewed.len(), 3);
    assert!(matches!(
        saved.reviewed["c|file"].state,
        ReviewEntryState::File
    ));
}

#[test]
fn unknown_entry_field_survives_a_mark_and_save() {
    // The app and the CLI share the store and can be at different versions; an older binary rewriting an entry must not strip a newer one's fields.
    let json = r#"{"reviewed":{"c1|a.txt":{"identity":"id-v1","state":{"kind":"file"},"reviewer":"ada"}}}"#;
    let parsed: StoredReviews = serde_json::from_str(json).unwrap();
    let mut store = ReviewStore::from_state(parsed);
    store.mark_reviewed("c1", "a.txt", "id-v2");

    let text = serde_json::to_string(&store.state).unwrap();
    assert!(
        text.contains(r#""reviewer":"ada""#),
        "unknown entry field must survive save: {text}"
    );
}

#[test]
fn unparseable_notes_survive_load_and_save() {
    // A note written by a newer version (unknown side variant here) must not be silently deleted by the next save; only parseable notes surface in the API.
    let json = r#"{"reviewed":{},"notes":[{"id":"n2","side":"inline","line":true},{"id":"n1","change_id":"c1","path":"a.txt","identity":"id","side":"new","line":1,"anchor_excerpt":"x","body":"check","created_at_ms":1,"updated_at_ms":1}]}"#;
    let parsed: StoredReviews = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.notes.len(), 2);

    let mut store = ReviewStore::from_state(parsed);
    assert_eq!(store.list_notes("c1", true).len(), 1);
    store.mark_reviewed("c1", "a.txt", "id");

    let text = serde_json::to_string(&store.state).unwrap();
    let round_tripped: StoredReviews = serde_json::from_str(&text).unwrap();
    assert_eq!(round_tripped.notes.len(), 2);
    assert!(
        text.contains(r#""side":"inline""#),
        "unknown note shape must be written back verbatim: {text}"
    );
}

#[test]
fn newer_version_note_fields_survive_save() {
    // A note that parses but carries fields from a newer version must keep them across a mutation + save by an older binary (app and CLI share the store and can be at different versions).
    let json = r#"{"reviewed":{},"notes":[{"id":"n1","change_id":"c1","path":"a.txt","identity":"id","side":"new","line":1,"anchor_excerpt":"x","body":"check","created_at_ms":1,"updated_at_ms":1,"severity":"high"}]}"#;
    let parsed: StoredReviews = serde_json::from_str(json).unwrap();
    let mut store = ReviewStore::from_state(parsed);
    store.resolve_note("n1").unwrap();

    let text = serde_json::to_string(&store.state).unwrap();
    assert!(
        text.contains(r#""severity":"high""#),
        "unknown note field must survive save: {text}"
    );
    assert!(text.contains(r#""resolved":true"#));
}

#[test]
fn note_crud_resolved_filtering_and_single_active_per_line() {
    let mut store = note_store();
    let first = store.add_note(anchor(), "check this");
    assert_eq!(first.id, "note-1");
    assert!(first.created_at_ms > 0);

    let mut moved_anchor = anchor();
    moved_anchor.anchor_excerpt = "newer line text".to_string();
    moved_anchor.anchor_context = vec!["newer line text".to_string()];
    let second = store.add_note(moved_anchor, "also check this");
    assert_eq!(second.id, first.id);
    assert_eq!(second.body, "also check this");
    assert_eq!(second.anchor_excerpt, "newer line text");
    assert_eq!(store.list_notes("c1", false).len(), 1);

    let updated = store.update_note(&second.id, "edited").unwrap();
    assert_eq!(updated.body, "edited");
    assert!(updated.updated_at_ms >= updated.created_at_ms);

    let resolved = store.resolve_note(&second.id).unwrap();
    assert!(resolved.resolved_at_ms.unwrap() >= resolved.created_at_ms);
    assert!(store.list_notes("c1", false).is_empty());

    let third = store.add_note(anchor(), "new active note");
    assert_eq!(third.id, "note-2");
    let active = store.list_notes("c1", false);
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, third.id);
    assert_eq!(store.list_notes("c1", true).len(), 2);

    assert!(store.delete_note(&third.id));
    assert!(store.list_notes("c1", false).is_empty());
}

#[test]
fn clear_all_drops_marks_and_notes_but_keeps_unknown_root_fields() {
    let json = r#"{"reviewed":{"c1|a.txt":{"identity":"id","state":{"kind":"file"}}},"future":{"kept":true}}"#;
    let parsed: StoredReviews = serde_json::from_str(json).unwrap();
    let mut store = ReviewStore::from_state(parsed);
    store.add_note(anchor(), "check this");
    assert_eq!(store.summary(), ReviewStoreSummary { marks: 1, notes: 1 });

    store.clear_all();

    assert_eq!(store.summary(), ReviewStoreSummary { marks: 0, notes: 0 });
    let text = serde_json::to_string(&store.state).unwrap();
    assert!(text.contains(r#""future":{"kept":true}"#), "{text}");
}
