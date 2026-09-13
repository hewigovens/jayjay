use std::fs;

use jayjay_core::{
    Repo, ReviewOutputFormat, mark_review_file, review_status_output, unmark_review_files,
};
use jayjay_primitives::{NoteSide, ReviewGroupState};
use jayjay_review::ReviewStore;
use jj_test::{add_two_group_edit, init_jj_repo, review_store_env, run_jj_in};

#[test]
fn agent_marks_groups_and_files_the_app_recognizes() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let store_path = temp_dir.path().join("review_store.json");
    let _guard = review_store_env(&store_path);
    let path = add_two_group_edit(&repo_path);

    let text = review_status_output(&repo_path, ReviewOutputFormat::Text).unwrap();
    let commit = text
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("Commit "))
        .unwrap()
        .to_owned();
    assert!(
        text.contains("unreviewed                review.txt\n"),
        "{text}"
    );
    assert!(
        text.contains("0 of 1 files reviewed (0 by agent), 0 partial"),
        "{text}"
    );

    assert!(mark_review_file(&repo_path, "missing.txt", None, NoteSide::New, &commit).is_err());
    assert!(mark_review_file(&repo_path, path, Some(3), NoteSide::New, &commit).is_err());

    let marked = mark_review_file(&repo_path, path, Some(1), NoteSide::New, &commit).unwrap();
    assert_eq!(marked, "Marked review.txt:1 reviewed (1/2 groups)\n");
    let file = &serde_json::from_str::<serde_json::Value>(
        &review_status_output(&repo_path, ReviewOutputFormat::Json).unwrap(),
    )
    .unwrap()["files"][0];
    assert_eq!(file["status"], "partial");
    assert_eq!(file["source"], "agent");
    assert_eq!(file["reviewed_groups"], 1);
    assert_eq!(file["total_groups"], 2);
    assert!(
        review_status_output(&repo_path, ReviewOutputFormat::Text)
            .unwrap()
            .contains("partial              1/2  review.txt  (agent)\n"),
        "{}",
        review_status_output(&repo_path, ReviewOutputFormat::Text).unwrap()
    );

    let marked = mark_review_file(&repo_path, path, Some(7), NoteSide::Old, &commit).unwrap();
    assert_eq!(marked, "Marked review.txt:7 reviewed (2/2 groups)\n");
    assert!(
        review_status_output(&repo_path, ReviewOutputFormat::Text)
            .unwrap()
            .contains("1 of 1 files reviewed (1 by agent), 0 partial")
    );

    let repo = Repo::open(&repo_path).expect("open repo");
    let detail = repo.show("@").expect("show working copy");
    let change_id = detail.info.change_id.id.clone();
    let hunk = detail
        .diff
        .iter()
        .find(|hunk| hunk.path == path)
        .expect("review hunk");
    let snapshot = jayjay_core::review_snapshot_from_hunk(hunk);
    let state = ReviewStore::load().file_review_state(
        &change_id,
        path,
        &hunk.review_identity,
        Some(&snapshot),
    );
    assert_eq!(
        state.group_states(),
        [ReviewGroupState::Reviewed, ReviewGroupState::Reviewed]
    );

    assert_eq!(
        unmark_review_files(&repo_path, Some(path)).unwrap(),
        "Unmarked review.txt\n"
    );
    assert!(
        review_status_output(&repo_path, ReviewOutputFormat::Text)
            .unwrap()
            .contains("0 of 1 files reviewed (0 by agent), 0 partial")
    );

    assert_eq!(
        mark_review_file(&repo_path, path, None, NoteSide::New, &commit).unwrap(),
        "Marked review.txt reviewed\n"
    );
    let parsed = serde_json::from_str::<serde_json::Value>(
        &review_status_output(&repo_path, ReviewOutputFormat::Json).unwrap(),
    )
    .unwrap();
    assert_eq!(parsed["schema_version"], 1);
    assert_eq!(parsed["change_id"], change_id);
    assert_eq!(parsed["reviewed"], 1);
    assert_eq!(parsed["total"], 1);
    assert_eq!(parsed["files"][0]["source"], "agent");
    assert!(
        review_status_output(&repo_path, ReviewOutputFormat::Text)
            .unwrap()
            .contains("1 of 1 files reviewed (1 by agent), 0 partial"),
        "{}",
        review_status_output(&repo_path, ReviewOutputFormat::Text).unwrap()
    );
    assert_eq!(
        unmark_review_files(&repo_path, None).unwrap(),
        "Unmarked 1 agent-reviewed files\n  review.txt\n"
    );

    ReviewStore::load().mark_reviewed(&change_id, path, &hunk.review_identity);
    assert!(unmark_review_files(&repo_path, Some(path)).is_err());
    assert_eq!(
        unmark_review_files(&repo_path, None).unwrap(),
        "Unmarked 0 agent-reviewed files\n"
    );
    assert!(
        review_status_output(&repo_path, ReviewOutputFormat::Text)
            .unwrap()
            .contains("1 of 1 files reviewed (0 by agent), 0 partial")
    );
}

#[test]
fn stale_inspections_are_cli_errors() {
    let temp = init_jj_repo();
    let repo = temp.path().join("repo");
    let store_path = temp.path().join("store").join("marks.json");
    let _guard = review_store_env(&store_path);
    let path = add_two_group_edit(&repo);
    let inspected = review_status_output(&repo, ReviewOutputFormat::Text)
        .unwrap()
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("Commit "))
        .unwrap()
        .to_owned();
    fs::write(repo.join(path), "uninspected behavior\n").unwrap();
    for line in [None, Some("1")] {
        let mut args = vec![
            "review",
            "mark",
            "--repo",
            repo.to_str().unwrap(),
            "--file",
            path,
            "--expected-commit",
            &inspected,
        ];
        if let Some(line) = line {
            args.extend(["--line", line]);
        }
        let outcome = jayjay_core::run_app_cli_command(
            &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            "test",
        )
        .unwrap();
        assert!(outcome.is_error());
        assert!(
            outcome.message.contains("changed since inspection"),
            "{}",
            outcome.message
        );
    }
    assert!(!store_path.exists());
    let current = review_status_output(&repo, ReviewOutputFormat::Text)
        .unwrap()
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("Commit "))
        .unwrap()
        .to_owned();
    assert_ne!(current, inspected);
    assert!(mark_review_file(&repo, path, None, NoteSide::New, "@").is_err());
    run_jj_in(&repo, &["new"]);
    fs::write(repo.join(path), "another change\n").unwrap();
    assert!(mark_review_file(&repo, path, None, NoteSide::New, &current).is_err());
    assert!(!store_path.exists());
}

#[test]
fn failed_mark_save_is_a_cli_error() {
    let temp = init_jj_repo();
    let repo = temp.path().join("repo");
    let store_path = temp.path().join("store").join("marks.json");
    let _guard = review_store_env(&store_path);
    let path = add_two_group_edit(&repo);
    let current = review_status_output(&repo, ReviewOutputFormat::Text)
        .unwrap()
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("Commit "))
        .unwrap()
        .to_owned();
    fs::write(store_path.parent().unwrap(), "blocks the store directory").unwrap();
    let args = [
        "review",
        "mark",
        "--repo",
        repo.to_str().unwrap(),
        "--file",
        path,
        "--expected-commit",
        &current,
    ];
    let outcome = jayjay_core::run_app_cli_command(
        &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        "test",
    )
    .unwrap();
    assert!(outcome.is_error());
    assert!(
        outcome.message.contains("Could not save review marks"),
        "{}",
        outcome.message
    );
}
