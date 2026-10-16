use std::fs;

use jayjay_core::{HunkType, Repo};
use jj_test::{current_op_id, init_jj_repo, run_jj, run_jj_in};

#[test]
fn interdiff_file_spans_commit_endpoints_and_deletion() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");

    run_jj_in(&repo_path, &["new", "-m", "edit a"]);
    fs::write(repo_path.join("hello.txt"), "a\n").expect("write a");
    run_jj_in(&repo_path, &["new", "-m", "edit b"]);
    fs::write(repo_path.join("hello.txt"), "b\n").expect("write b");
    run_jj_in(&repo_path, &["new", "-m", "edit c"]);
    fs::write(repo_path.join("hello.txt"), "c\n").expect("write c");
    run_jj_in(&repo_path, &["st"]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let history = repo.file_history("hello.txt").expect("file history");
    let commit_id = |description: &str| {
        history
            .iter()
            .find(|change| change.description.trim() == description)
            .unwrap_or_else(|| panic!("missing {description:?} in history"))
            .commit_id
            .id
            .clone()
    };

    let across = repo
        .interdiff_file(&commit_id("edit a"), &commit_id("edit c"), "hello.txt")
        .expect("interdiff a to c");
    assert_eq!(across.old.content.as_deref(), Some("a\n"));
    assert_eq!(across.new.content.as_deref(), Some("c\n"));
    assert_eq!(across.hunk_type, HunkType::Modified);

    run_jj_in(&repo_path, &["new", "-m", "delete"]);
    fs::remove_file(repo_path.join("hello.txt")).expect("delete file");
    run_jj_in(&repo_path, &["st"]);

    let repo = Repo::open(&repo_path).expect("reopen repo");
    let history = repo
        .file_history("hello.txt")
        .expect("file history after deletion");
    let deleted = history
        .iter()
        .find(|change| change.description.trim() == "delete")
        .expect("deletion appears in file history")
        .commit_id
        .id
        .clone();

    let removed = repo
        .interdiff_file(&commit_id("edit c"), &deleted, "hello.txt")
        .expect("interdiff c to deletion");
    assert_eq!(removed.hunk_type, HunkType::Removed);
    assert_eq!(removed.old.content.as_deref(), Some("c\n"));
    assert!(removed.new.content.is_none());
}

#[test]
fn interdiff_file_resolves_divergent_commit_id_endpoints_to_their_own_content() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");
    fs::write(repo_path.join("hello.txt"), "right\n").expect("write right");
    run_jj(&["-R", repo_str, "describe", "-m", "right version"]);
    let base_op = current_op_id(&repo_path);

    fs::write(repo_path.join("hello.txt"), "left\n").expect("write left");
    run_jj(&["-R", repo_str, "describe", "-m", "left version"]);
    run_jj(&[
        "-R",
        repo_str,
        "--at-op",
        &base_op,
        "describe",
        "-m",
        "right fork",
    ]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let changes = repo.log("all()").expect("load log");
    let left = changes
        .iter()
        .find(|change| change.is_divergent && change.description.trim() == "left version")
        .expect("left divergent sibling");
    let right = changes
        .iter()
        .find(|change| change.is_divergent && change.description.trim() == "right fork")
        .expect("right divergent sibling");
    assert_eq!(left.change_id.id, right.change_id.id);
    assert_ne!(left.commit_id.id, right.commit_id.id);

    let hunk = repo
        .interdiff_file(&left.commit_id.id, &right.commit_id.id, "hello.txt")
        .expect("interdiff divergent endpoints");
    assert_eq!(hunk.old.content.as_deref(), Some("left\n"));
    assert_eq!(hunk.new.content.as_deref(), Some("right\n"));
}
