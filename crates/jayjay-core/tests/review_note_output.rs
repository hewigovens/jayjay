use std::fs;
use std::path::Path;

use jayjay_core::{
    Repo, ReviewNoteOutputFormat, add_review_note, resolve_review_note, review_notes_output,
};
use jayjay_primitives::NoteSide;
use jj_test::{init_jj_repo, review_store_env};
use serde_json::json;

const HELLO_PATH: &str = "hello.txt";
const HELLO_CONTENT: &str = "hello from jayjay\nplease check this\n";

fn assert_success(result: jayjay_core::CoreResult<String>) -> String {
    result.unwrap_or_else(|error| panic!("expected success: {error}"))
}

fn edit_hello(repo_path: &Path) {
    fs::write(repo_path.join(HELLO_PATH), HELLO_CONTENT).expect("edit file");
}

fn json_notes(repo_path: &Path, include_resolved: bool) -> serde_json::Value {
    let output = assert_success(review_notes_output(
        repo_path,
        ReviewNoteOutputFormat::Json,
        include_resolved,
    ));
    serde_json::from_str(&output).expect("json notes")
}

fn text_notes(repo_path: &Path) -> String {
    assert_success(review_notes_output(
        repo_path,
        ReviewNoteOutputFormat::Text,
        false,
    ))
}

#[test]
fn review_notes_json_and_resolve_note() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let store_path = temp_dir.path().join("review_store.json");
    let _guard = review_store_env(&store_path);

    edit_hello(&repo_path);
    let repo = Repo::open(&repo_path).expect("open repo");
    repo.refresh_working_copy().expect("snapshot");
    let detail = repo.show("@").expect("show working copy");
    let change_id = detail.info.change_id.id;
    let hunk = detail
        .diff
        .into_iter()
        .find(|hunk| hunk.path == HELLO_PATH)
        .expect("hello hunk");
    let note_id = "note-test-1";
    let store_json = json!({
        "reviewed": {},
        "notes": [{
            "id": note_id,
            "change_id": change_id,
            "path": hunk.path,
            "identity": hunk.review_identity,
            "side": "new",
            "line": 2,
            "anchor_excerpt": "please check this",
            "anchor_context": ["please check this"],
            "body": "Please check this edge case",
            "created_at_ms": 1000,
            "updated_at_ms": 1000
        }]
    });
    fs::write(&store_path, serde_json::to_vec(&store_json).unwrap()).expect("write store");

    let parsed = json_notes(&repo_path, false);
    assert_eq!(parsed["schema_version"], 1);
    assert_eq!(parsed["notes"][0]["note"]["id"], note_id);
    assert_eq!(parsed["notes"][0]["status"], "current");

    assert!(resolve_review_note(&repo_path, "note-from-another-repo").is_err());

    let resolved = assert_success(resolve_review_note(&repo_path, note_id));
    assert_eq!(resolved, "Resolved review note note-test-1\n");

    let parsed = json_notes(&repo_path, false);
    assert_eq!(parsed["notes"].as_array().unwrap().len(), 0);

    let parsed = json_notes(&repo_path, true);
    assert_eq!(parsed["notes"][0]["status"], "resolved");
}

#[test]
fn add_note_anchors_a_changed_line_and_rejects_unchanged_lines() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let store_path = temp_dir.path().join("review_store.json");
    let _guard = review_store_env(&store_path);

    edit_hello(&repo_path);

    let added = assert_success(add_review_note(
        &repo_path,
        HELLO_PATH,
        2,
        NoteSide::New,
        "added from the CLI",
    ));
    assert!(added.starts_with("Added review note "), "{added}");

    let parsed = json_notes(&repo_path, false);
    assert_eq!(parsed["notes"][0]["status"], "current");
    assert_eq!(parsed["notes"][0]["note"]["body"], "added from the CLI");
    assert_eq!(parsed["notes"][0]["note"]["line"], 2);

    let text = text_notes(&repo_path);
    assert!(text.contains("hello.txt:2 [current]"), "header: {text}");
    assert!(
        text.contains("  anchor: please check this"),
        "anchor line: {text}"
    );
    assert!(text.contains("  added from the CLI"), "body: {text}");

    for (line, side) in [(99, NoteSide::New), (1, NoteSide::Old)] {
        let rejected = add_review_note(&repo_path, HELLO_PATH, line, side, "should not exist");
        assert!(
            rejected.is_err(),
            "expected rejection for line {line} side {side:?}"
        );
    }
}
