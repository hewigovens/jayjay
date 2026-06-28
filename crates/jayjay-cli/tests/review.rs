use std::fs;
use std::process::Command;

use jayjay_core::Repo;
use jj_test::init_jj_repo;
use serde_json::json;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jayjay"))
}

fn output(command: &mut Command) -> std::process::Output {
    command.output().expect("run jayjay")
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

    let notes = output(
        cli()
            .args([
                "review",
                "notes",
                "--repo",
                repo_path.to_str().unwrap(),
                "--format",
                "json",
            ])
            .env("JAYJAY_REVIEW_STORE_PATH", &store_path),
    );
    assert!(
        notes.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&notes.stdout),
        String::from_utf8_lossy(&notes.stderr)
    );
    let parsed: serde_json::Value = serde_json::from_slice(&notes.stdout).expect("json output");
    assert_eq!(parsed["schema_version"], 1);
    assert_eq!(parsed["notes"][0]["note"]["id"], note_id);
    assert_eq!(parsed["notes"][0]["status"], "current");

    // A note that doesn't belong to this repo's working-copy change must not resolve.
    let foreign = output(
        cli()
            .args([
                "review",
                "resolve-note",
                "note-from-another-repo",
                "--repo",
                repo_path.to_str().unwrap(),
            ])
            .env("JAYJAY_REVIEW_STORE_PATH", &store_path),
    );
    assert!(!foreign.status.success());

    let resolved = output(
        cli()
            .args([
                "review",
                "resolve-note",
                note_id,
                "--repo",
                repo_path.to_str().unwrap(),
            ])
            .env("JAYJAY_REVIEW_STORE_PATH", &store_path),
    );
    assert!(
        resolved.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&resolved.stdout),
        String::from_utf8_lossy(&resolved.stderr)
    );

    let hidden = output(
        cli()
            .args([
                "review",
                "notes",
                "--repo",
                repo_path.to_str().unwrap(),
                "--format",
                "json",
            ])
            .env("JAYJAY_REVIEW_STORE_PATH", &store_path),
    );
    let parsed: serde_json::Value = serde_json::from_slice(&hidden.stdout).expect("json output");
    assert_eq!(parsed["notes"].as_array().unwrap().len(), 0);

    let included = output(
        cli()
            .args([
                "review",
                "notes",
                "--repo",
                repo_path.to_str().unwrap(),
                "--format",
                "json",
                "--include-resolved",
            ])
            .env("JAYJAY_REVIEW_STORE_PATH", &store_path),
    );
    let parsed: serde_json::Value = serde_json::from_slice(&included.stdout).expect("json output");
    assert_eq!(parsed["notes"][0]["status"], "resolved");
}
