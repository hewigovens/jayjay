use std::fs;

use jayjay_core::{CliCommandOutcome, run_app_cli_command};
use jj_test::{init_jj_repo, review_store_env};

fn run(arguments: &[&str]) -> CliCommandOutcome {
    let arguments: Vec<String> = arguments.iter().map(|value| value.to_string()).collect();
    run_app_cli_command(&arguments, "0.0.0").expect("handled")
}

#[test]
fn review_round_trip_formats_outcomes_shared_by_both_shells() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let store_path = temp_dir.path().join("review_store.json");
    let _env = review_store_env(&store_path);

    fs::write(
        repo_path.join("hello.txt"),
        "hello from jayjay\nplease check this\n",
    )
    .expect("edit file");
    let repo = repo_path.to_str().expect("utf8 repo path");

    let added = run(&[
        "review",
        "add-note",
        "--repo",
        repo,
        "--file",
        "hello.txt",
        "--line",
        "2",
        "-m",
        "- check this edge",
    ]);
    assert!(!added.is_error(), "{}", added.message);
    assert!(
        added.message.starts_with("Added review note "),
        "{}",
        added.message
    );

    let notes = run(&["review", "notes", "--repo", repo, "--format", "json"]);
    assert_eq!(notes.exit_code, 0);
    let json: serde_json::Value = serde_json::from_str(&notes.message).expect("valid json");
    assert_eq!(json["notes"][0]["status"], "current");
    assert_eq!(json["notes"][0]["note"]["body"], "- check this edge");
    let note_id = json["notes"][0]["note"]["id"]
        .as_str()
        .expect("note id")
        .to_owned();

    let resolved = run(&["review", "resolve-note", "--repo", repo, &note_id]);
    assert_eq!(resolved.exit_code, 0);
    assert_eq!(
        resolved.message,
        format!("Resolved review note {note_id}\n")
    );

    let rejected = run(&[
        "review",
        "add-note",
        "--repo",
        repo,
        "--file",
        "hello.txt",
        "--line",
        "99",
        "-m",
        "nope",
    ]);
    assert_eq!(rejected.exit_code, 1);
    assert!(rejected.is_error());
    assert!(
        rejected.message.starts_with("error: "),
        "{}",
        rejected.message
    );
}
