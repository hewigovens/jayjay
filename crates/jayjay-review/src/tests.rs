use super::*;
use crate::store::{StoredReviews, key};
use crate::test_util::{FixedClock, SequentialIds};

fn make_store() -> ReviewStore {
    ReviewStore::in_memory()
}

fn note_store(now: i64) -> ReviewStore {
    ReviewStore::in_memory_with_sources(Box::new(SequentialIds::new()), Box::new(FixedClock(now)))
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
fn json_load_drops_legacy_and_save_round_trips() {
    let json = r#"{"reviewed":{"c|legacy":12.34,"c|new":{"identity":"id1","file_marked":true,"hunks":[1,3]}}}"#;
    let parsed: StoredReviews = serde_json::from_str(json).unwrap();
    assert!(!parsed.reviewed.contains_key("c|legacy"));
    let e = &parsed.reviewed["c|new"];
    assert_eq!(e.identity, "id1");
    assert!(e.file_marked);
    assert_eq!(e.hunks, vec![1, 3]);
    assert!(parsed.notes.is_empty());

    let text = serde_json::to_string(&parsed).unwrap();
    let saved: StoredReviews = serde_json::from_str(&text).unwrap();
    assert_eq!(saved.reviewed.len(), 1);
    assert!(saved.notes.is_empty());
}

#[test]
fn unparseable_notes_survive_load_and_save() {
    // A note written by a newer version (unknown side variant here) must not be silently deleted by the next save; only parseable notes surface in the API.
    let json = r#"{"reviewed":{},"notes":[{"id":"n2","side":"inline","line":true},{"id":"n1","change_id":"c1","path":"a.txt","identity":"id","side":"new","line":1,"anchor_excerpt":"x","body":"check","created_at_ms":1,"updated_at_ms":1}]}"#;
    let parsed: StoredReviews = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.notes.len(), 2);

    let mut store = ReviewStore::from_state(parsed, true);
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
    let mut store = ReviewStore::from_state(parsed, true);
    store.resolve_note("n1").unwrap();

    let text = serde_json::to_string(&store.state).unwrap();
    assert!(
        text.contains(r#""severity":"high""#),
        "unknown note field must survive save: {text}"
    );
    assert!(text.contains(r#""resolved":true"#));
}

#[test]
fn refresh_from_disk_keeps_state_when_file_deleted() {
    // An externally deleted store file must not turn the next mutation into a save of only that mutation; the in-memory state is the best recovery.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("review_store.json");
    let mut store = ReviewStore::load_from(path.clone());
    store.mark_reviewed("c1", "a.txt", "id-v1");
    std::fs::remove_file(&path).unwrap();

    store.refresh_from_disk();
    store.mark_reviewed("c1", "b.txt", "id-v1");

    let reloaded = ReviewStore::load_from(path);
    assert!(reloaded.is_reviewed("c1", "a.txt", "id-v1"));
    assert!(reloaded.is_reviewed("c1", "b.txt", "id-v1"));
}

#[test]
fn refresh_from_disk_prevents_clobbering_external_writes() {
    // A long-lived store (GPUI holds one for the process lifetime) must not rewrite the file from its stale snapshot after another process (CLI, SwiftUI shell) added a note or mark.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("review_store.json");

    let mut long_lived = ReviewStore::load_from(path.clone());
    long_lived.mark_reviewed("c1", "a.txt", "id-v1");

    let mut other_process = ReviewStore::load_from(path.clone());
    other_process.add_note(anchor(), "written by the CLI");

    long_lived.refresh_from_disk();
    long_lived.mark_reviewed("c1", "b.txt", "id-v1");

    let reloaded = ReviewStore::load_from(path);
    assert_eq!(reloaded.list_notes("c1", true).len(), 1);
    assert!(reloaded.is_reviewed("c1", "a.txt", "id-v1"));
    assert!(reloaded.is_reviewed("c1", "b.txt", "id-v1"));
}

#[test]
fn refresh_if_stale_picks_up_external_writes_and_skips_when_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("review_store.json");

    let mut long_lived = ReviewStore::load_from(path.clone());
    long_lived.mark_reviewed("c1", "a.txt", "id-v1");

    long_lived.refresh_if_stale();
    assert!(long_lived.is_reviewed("c1", "a.txt", "id-v1"));

    let mut other_process = ReviewStore::load_from(path.clone());
    other_process.mark_reviewed("c1", "b.txt", "id-v1");

    long_lived.refresh_if_stale();
    assert!(long_lived.is_reviewed("c1", "b.txt", "id-v1"));
    assert!(long_lived.is_reviewed("c1", "a.txt", "id-v1"));
}

#[test]
fn note_crud_resolved_filtering_and_single_active_per_line() {
    let mut store = note_store(100);
    let first = store.add_note(anchor(), "check this");
    assert_eq!(first.id, "note-1");
    assert_eq!(first.created_at_ms, 100);

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

    store.resolve_note(&second.id).unwrap();
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
fn write_to_persists_atomically_and_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested").join("review_store.json");

    let mut s = make_store();
    s.mark_reviewed("c1", "a.txt", "id-v1");
    s.write_to(&path).unwrap();

    let stray_temp = std::fs::read_dir(path.parent().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .any(|entry| entry.file_name().to_string_lossy().contains("tmp"));
    assert!(!stray_temp);

    let text = std::fs::read_to_string(&path).unwrap();
    let state: StoredReviews = serde_json::from_str(&text).unwrap();
    let reloaded = ReviewStore::from_state(state, true);
    assert!(reloaded.is_reviewed("c1", "a.txt", "id-v1"));
}

#[test]
fn write_to_replaces_existing_file_without_clobbering_notes_or_extra_keys() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("review_store.json");
    let json = r#"{"reviewed":{},"notes":[{"id":"n1","change_id":"c1","path":"a.txt","identity":"id","side":"new","line":1,"anchor_excerpt":"x","body":"check","created_at_ms":1,"updated_at_ms":1}],"future":true}"#;
    std::fs::write(&path, json).unwrap();

    let state = ReviewStore::load_path(path.clone());
    let mut store = ReviewStore::from_state(state, true);
    store.mark_reviewed("c1", "b.txt", "id-v1");
    store.write_to(&path).unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    let state: StoredReviews = serde_json::from_str(&text).unwrap();
    assert_eq!(state.reviewed.len(), 1);
    assert_eq!(state.notes.len(), 1);
    assert_eq!(state.extra["future"], true);
    assert!(state.reviewed.contains_key(&key("c1", "b.txt")));
}

#[test]
fn corrupt_file_is_preserved_not_silently_wiped_on_load() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("review_store.json");
    // Truncated JSON, the shape an interrupted write would leave behind.
    std::fs::write(&path, r#"{"reviewed":{"c|a.txt":{"identi"#).unwrap();

    let state = ReviewStore::load_path(path.clone());

    assert!(state.reviewed.is_empty());
    assert!(!path.exists());
    assert!(path.with_extension("json.corrupt").exists());
}

#[test]
fn missing_file_loads_empty_without_creating_corrupt_sibling() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("review_store.json");

    let state = ReviewStore::load_path(path.clone());

    assert!(state.reviewed.is_empty());
    assert!(!path.with_extension("json.corrupt").exists());
}
