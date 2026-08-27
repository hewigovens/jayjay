use super::*;
use crate::store::key;

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
fn write_to_persists_atomically_and_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested").join("review_store.json");

    let mut store = make_store();
    store.mark_reviewed("c1", "a.txt", "id-v1");
    store.write_to(&path).unwrap();

    let stray_temp = std::fs::read_dir(path.parent().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .any(|entry| entry.file_name().to_string_lossy().contains("tmp"));
    assert!(!stray_temp);

    let text = std::fs::read_to_string(&path).unwrap();
    let state: StoredReviews = serde_json::from_str(&text).unwrap();
    let reloaded = ReviewStore::from_state(state);
    assert!(reloaded.is_reviewed("c1", "a.txt", "id-v1"));
}

#[test]
fn write_to_replaces_existing_file_without_clobbering_notes_or_extra_keys() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("review_store.json");
    let json = r#"{"reviewed":{},"notes":[{"id":"n1","change_id":"c1","path":"a.txt","identity":"id","side":"new","line":1,"anchor_excerpt":"x","body":"check","created_at_ms":1,"updated_at_ms":1}],"future":true}"#;
    std::fs::write(&path, json).unwrap();

    let state = ReviewStore::load_path(path.clone());
    let mut store = ReviewStore::from_state(state);
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
