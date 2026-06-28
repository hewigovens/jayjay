use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use jayjay_core::Repo;
use jj_test::{init_jj_repo, json_stdout};
use serde_json::json;

/// Runs `jayjay review <args>` against `repo` with the store redirected to `store`.
fn review_cmd(repo: &Path, store: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jayjay"))
        .arg("review")
        .args(args)
        .args(["--repo", repo.to_str().unwrap()])
        .env("JAYJAY_REVIEW_STORE_PATH", store)
        .output()
        .expect("run jayjay")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn review_notes_json_and_resolve_note_do_not_launch_gui() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let store_path = temp_dir.path().join("review_store.json");

    fs::write(
        repo_path.join("hello.txt"),
        "hello from jayjay\nplease check this\n",
    )
    .expect("edit file");
    let repo = Repo::open(&repo_path).expect("open repo");
    repo.refresh_working_copy().expect("snapshot");
    let detail = repo.show("@").expect("show working copy");
    let change_id = detail.info.change_id.id;
    let hunk = detail
        .diff
        .into_iter()
        .find(|hunk| hunk.path == "hello.txt")
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

    let notes = review_cmd(&repo_path, &store_path, &["notes", "--format", "json"]);
    assert_success(&notes);
    let parsed = json_stdout(&notes);
    assert_eq!(parsed["schema_version"], 1);
    assert_eq!(parsed["notes"][0]["note"]["id"], note_id);
    assert_eq!(parsed["notes"][0]["status"], "current");

    // A note that doesn't belong to this repo's working-copy change must not resolve.
    let foreign = review_cmd(
        &repo_path,
        &store_path,
        &["resolve-note", "note-from-another-repo"],
    );
    assert!(!foreign.status.success());

    let resolved = review_cmd(&repo_path, &store_path, &["resolve-note", note_id]);
    assert_success(&resolved);

    let hidden = review_cmd(&repo_path, &store_path, &["notes", "--format", "json"]);
    assert_eq!(json_stdout(&hidden)["notes"].as_array().unwrap().len(), 0);

    let included = review_cmd(
        &repo_path,
        &store_path,
        &["notes", "--format", "json", "--include-resolved"],
    );
    assert_eq!(json_stdout(&included)["notes"][0]["status"], "resolved");
}

#[test]
fn add_note_anchors_a_changed_line_and_rejects_unchanged_lines() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let store_path = temp_dir.path().join("review_store.json");

    fs::write(
        repo_path.join("hello.txt"),
        "hello from jayjay\nplease check this\n",
    )
    .expect("edit file");

    let added = review_cmd(
        &repo_path,
        &store_path,
        &[
            "add-note",
            "--file",
            "hello.txt",
            "--line",
            "2",
            "--message",
            "added from the CLI",
        ],
    );
    assert_success(&added);

    // The note must reconcile Current through the same pipeline the GUI and `notes` use.
    let notes = review_cmd(&repo_path, &store_path, &["notes", "--format", "json"]);
    let parsed = json_stdout(&notes);
    assert_eq!(parsed["notes"][0]["status"], "current");
    assert_eq!(parsed["notes"][0]["note"]["body"], "added from the CLI");
    assert_eq!(parsed["notes"][0]["note"]["line"], 2);

    // The default text format is agent-consumable as-is: path:line header, anchor line, and the full body indented beneath.
    let text = review_cmd(&repo_path, &store_path, &["notes"]);
    assert_success(&text);
    let stdout = String::from_utf8_lossy(&text.stdout);
    assert!(stdout.contains("hello.txt:2 [current]"), "header: {stdout}");
    assert!(
        stdout.contains("  anchor: please check this"),
        "anchor line: {stdout}"
    );
    assert!(stdout.contains("  added from the CLI"), "body: {stdout}");

    // Neither an out-of-range line nor the old side of an added file is a changed line; anchoring must fail loudly instead of writing a note that would immediately report stale.
    for (line, side) in [("99", "new"), ("1", "old")] {
        let rejected = review_cmd(
            &repo_path,
            &store_path,
            &[
                "add-note",
                "--file",
                "hello.txt",
                "--line",
                line,
                "--side",
                side,
                "--message",
                "should not exist",
            ],
        );
        assert!(
            !rejected.status.success(),
            "expected rejection for line {line} side {side}"
        );
    }
}
