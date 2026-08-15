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

fn at_commit_id(repo_path: &std::path::Path) -> String {
    let output = run_jj_in(
        repo_path,
        &[
            "--ignore-working-copy",
            "log",
            "-r",
            "@",
            "--no-graph",
            "-T",
            "commit_id",
        ],
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn workspace_named<'a>(
    workspaces: &'a [jayjay_core::WorkspaceInfo],
    name: &str,
) -> &'a jayjay_core::WorkspaceInfo {
    workspaces
        .iter()
        .find(|workspace| workspace.name == name)
        .unwrap_or_else(|| panic!("missing workspace {name}"))
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
fn workspace_list_includes_committed_metadata_without_snapshotting() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    run_jj_in(&repo_path, &["describe", "-m", "default workspace work"]);
    std::fs::write(repo_path.join("default-only.txt"), "default\n").expect("write default file");
    run_jj_in(&repo_path, &["describe", "-m", "default workspace work"]);

    let dest = temp_dir.path().join("repo-feature");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("workspace add");
    std::fs::write(dest.join("feature.txt"), "feature\n").expect("write feature file");
    run_jj_in(&dest, &["describe", "-m", "feature workspace work"]);

    run_jj_in(
        &repo_path,
        &["config", "set", "--repo", "snapshot.max-new-file-size", "1"],
    );
    let op_before = current_op_id_ignoring_working_copy(&repo_path);
    std::fs::write(repo_path.join("too-large-to-snapshot"), "xx").expect("write untracked file");
    std::fs::write(dest.join("too-large-to-snapshot"), "xx").expect("write feature untracked");

    let workspaces = repo
        .workspace_list()
        .expect("enriched list must not snapshot");
    assert_eq!(
        current_op_id_ignoring_working_copy(&repo_path),
        op_before,
        "enriched workspace list must not create a snapshot operation"
    );

    let default = workspace_named(&workspaces, "default");
    assert!(default.is_current);
    assert_eq!(default.description, "default workspace work");
    assert!(default.timestamp_millis.is_some());
    assert!(!default.wc_commit_id.is_empty());
    assert_eq!(
        default.changed_file_count,
        Some(
            repo.show_summary(&default.wc_commit_id)
                .expect("default summary")
                .diff
                .len() as u32
        )
    );

    let feature = workspace_named(&workspaces, "feature");
    assert!(!feature.is_current);
    assert_eq!(feature.description, "feature workspace work");
    assert!(feature.path.ends_with("repo-feature") || feature.path.contains("repo-feature"));
    let feature_summary = repo
        .show_summary(&feature.wc_commit_id)
        .expect("feature summary");
    assert_eq!(
        feature.changed_file_count,
        Some(feature_summary.diff.len() as u32)
    );
    assert!(
        feature_summary
            .diff
            .iter()
            .any(|hunk| hunk.path == "feature.txt"),
        "committed feature.txt must be in the WC-vs-parent diff: {:?}",
        feature_summary
            .diff
            .iter()
            .map(|hunk| &hunk.path)
            .collect::<Vec<_>>()
    );
}

#[test]
fn workspace_list_file_count_is_committed_tree_not_dirty_disk() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let dest = temp_dir.path().join("repo-feature");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("workspace add");
    std::fs::write(dest.join("committed.txt"), "committed\n").expect("write committed file");
    run_jj_in(&dest, &["describe", "-m", "committed feature file"]);

    let before = workspace_named(&repo.workspace_list().expect("list"), "feature")
        .changed_file_count
        .expect("count");

    std::fs::write(dest.join("dirty-only.txt"), "dirty\n").expect("write dirty file");
    // Do not run jj in the feature workspace after this: a snapshot would fold dirty-only.txt into @.

    let after = workspace_named(&repo.workspace_list().expect("list after dirty"), "feature")
        .changed_file_count
        .expect("count after dirty");
    assert_eq!(
        after, before,
        "dirty untracked files must not change the committed @-vs-parent file count"
    );
}

#[test]
fn workspace_list_sorts_default_before_named_workspaces() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    repo.workspace_add(
        temp_dir.path().join("repo-older").to_str().expect("utf8"),
        "older",
        "",
    )
    .expect("add older");
    repo.workspace_add(
        temp_dir.path().join("repo-newer").to_str().expect("utf8"),
        "newer",
        "",
    )
    .expect("add newer");

    let names: Vec<String> = repo
        .workspace_list()
        .expect("list")
        .into_iter()
        .map(|workspace| workspace.name)
        .collect();
    assert_eq!(names.first().map(String::as_str), Some("default"));
    assert!(names.contains(&"older".to_owned()));
    assert!(names.contains(&"newer".to_owned()));
}

#[test]
fn workspace_show_changes_does_not_edit_or_snapshot() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let dest = temp_dir.path().join("repo-feature");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("workspace add");
    std::fs::write(dest.join("feature.txt"), "feature\n").expect("write feature file");
    run_jj_in(&dest, &["describe", "-m", "feature workspace work"]);

    run_jj_in(
        &repo_path,
        &["config", "set", "--repo", "snapshot.max-new-file-size", "1"],
    );
    std::fs::write(repo_path.join("too-large-to-snapshot"), "xx").expect("write untracked file");
    std::fs::write(dest.join("too-large-to-snapshot"), "xx").expect("write feature untracked");

    let default_at_before = at_commit_id(&repo_path);
    let feature_at_before = at_commit_id(&dest);
    let op_before = current_op_id_ignoring_working_copy(&repo_path);

    let detail = repo
        .workspace_show_changes("feature")
        .expect("show other workspace changes");
    assert!(
        detail.diff.iter().any(|hunk| hunk.path == "feature.txt"),
        "Show Changes must use the other workspace's committed @ vs parent"
    );
    assert_eq!(detail.info.commit_id.id, feature_at_before);

    assert_eq!(at_commit_id(&repo_path), default_at_before);
    assert_eq!(at_commit_id(&dest), feature_at_before);
    assert_eq!(
        current_op_id_ignoring_working_copy(&repo_path),
        op_before,
        "inspecting another workspace must not snapshot or edit @"
    );
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

#[test]
fn workspace_list_sorts_default_first_then_timestamp_descending() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let older = temp_dir.path().join("older");
    repo.workspace_add(older.to_str().expect("utf8 dest"), "older", "")
        .expect("older workspace");
    std::thread::sleep(std::time::Duration::from_millis(20));
    run_jj_in(&older, &["describe", "-m", "older work"]);

    std::thread::sleep(std::time::Duration::from_millis(20));
    let newer = temp_dir.path().join("newer");
    repo.workspace_add(newer.to_str().expect("utf8 dest"), "newer", "")
        .expect("newer workspace");
    run_jj_in(&newer, &["describe", "-m", "newer work"]);

    let listed = repo.workspace_list().expect("list");
    let names: Vec<_> = listed.iter().map(|ws| ws.name.as_str()).collect();
    assert_eq!(names[0], "default", "{names:?}");
    let older_pos = names
        .iter()
        .position(|name| *name == "older")
        .expect("older");
    let newer_pos = names
        .iter()
        .position(|name| *name == "newer")
        .expect("newer");
    assert!(
        newer_pos < older_pos,
        "newer @ should sort before older: {names:?}"
    );
}
