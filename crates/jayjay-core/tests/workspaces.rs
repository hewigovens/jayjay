//! Workspace add/list/forget behavior over jj, including operand safety for option-shaped destinations and workspace names.

use jayjay_core::Repo;
use jj_test::{init_jj_repo, run_jj_in};

fn workspace_names(repo: &Repo) -> Vec<String> {
    repo.workspace_list()
        .expect("workspace list")
        .into_iter()
        .map(|ws| ws.name)
        .collect()
}

fn current_op_id_ignoring_working_copy(repo_path: &std::path::Path) -> String {
    let output = run_jj_in(
        repo_path,
        &[
            "--ignore-working-copy",
            "op",
            "log",
            "--no-graph",
            "--limit",
            "1",
        ],
    );
    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .expect("current operation id")
        .to_owned()
}

#[test]
fn workspace_add_and_forget_roundtrip() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let dest = temp_dir.path().join("repo-feature");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("workspace add");

    let names = workspace_names(&repo);
    assert!(names.contains(&"feature".to_owned()), "{names:?}");
    let workspaces = repo.workspace_list().expect("workspace list");
    let current = workspaces.iter().find(|ws| ws.is_current).expect("current");
    assert_eq!(current.name, "default", "adding must not switch workspaces");

    repo.workspace_forget("feature").expect("workspace forget");
    let names = workspace_names(&repo);
    assert!(!names.contains(&"feature".to_owned()), "{names:?}");
}

#[test]
fn workspace_list_does_not_snapshot_the_working_copy() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    run_jj_in(
        &repo_path,
        &["config", "set", "--repo", "snapshot.max-new-file-size", "1"],
    );
    let op_before = current_op_id_ignoring_working_copy(&repo_path);
    std::fs::write(repo_path.join("too-large-to-snapshot"), "xx").expect("write untracked file");

    let workspaces = repo
        .workspace_list()
        .expect("read-only workspace enumeration must not snapshot");
    let current = workspaces.iter().find(|ws| ws.is_current).expect("current");
    assert_eq!(
        std::fs::canonicalize(&current.path).expect("canonical workspace path"),
        std::fs::canonicalize(&repo_path).expect("canonical repo path")
    );
    assert_eq!(
        current_op_id_ignoring_working_copy(&repo_path),
        op_before,
        "workspace enumeration must not create a snapshot operation"
    );
}

#[test]
fn workspace_add_duplicate_name_errors() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let first = temp_dir.path().join("ws-one");
    let second = temp_dir.path().join("ws-two");
    repo.workspace_add(first.to_str().expect("utf8 dest"), "feature", "")
        .expect("first add");
    let err = repo
        .workspace_add(second.to_str().expect("utf8 dest"), "feature", "")
        .expect_err("duplicate workspace name must error");
    assert!(err.to_string().contains("already exists"), "{err}");
}

/// An option-shaped relative destination must reach jj as a literal path; without the `--` separator jj parses `-hostile` as flags and the add fails.
#[test]
fn workspace_add_treats_option_shaped_destination_as_literal_path() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    repo.workspace_add("-hostile", "hostile", "")
        .expect("option-shaped destination must be treated as a path");

    assert!(
        repo_path.join("-hostile").join(".jj").exists(),
        "workspace directory named -hostile should exist"
    );
    let names = workspace_names(&repo);
    assert!(names.contains(&"hostile".to_owned()), "{names:?}");

    repo.workspace_forget("hostile")
        .expect("forget with operand separator");
    let names = workspace_names(&repo);
    assert!(!names.contains(&"hostile".to_owned()), "{names:?}");
}

/// SwiftUI passes raw sheet input straight through, so core must enforce the name rules itself.
#[test]
fn workspace_add_rejects_invalid_names_in_core() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    for bad in ["../evil", "--name", "a/b", "a b", "@"] {
        let dest = temp_dir.path().join("dest");
        let err = repo
            .workspace_add(dest.to_str().expect("utf8 dest"), bad, "")
            .expect_err("invalid name must be rejected in core");
        assert!(err.to_string().contains("invalid workspace name"), "{err}");
        assert!(!dest.exists(), "{bad}: nothing may be created");
    }
    assert_eq!(workspace_names(&repo).len(), 1, "only the default remains");
}

#[test]
fn workspace_add_rejects_option_shaped_revision() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let dest = temp_dir.path().join("dest");
    let err = repo
        .workspace_add(dest.to_str().expect("utf8 dest"), "feature", "--config=x=y")
        .expect_err("option-shaped revision must be rejected");
    assert!(err.to_string().contains("invalid revision"), "{err}");
    assert!(!dest.exists());
}
