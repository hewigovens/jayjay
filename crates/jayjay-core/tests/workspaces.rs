use jayjay_core::{Repo, WorkspaceInfo};
use jj_lib::ref_name::WorkspaceName;
use jj_lib::workspace_store::{SimpleWorkspaceStore, WorkspaceStore as _};
use jj_test::{init_jj_repo, run_jj_in};

fn workspace_names(repo: &Repo) -> Vec<String> {
    repo.workspace_list()
        .expect("workspace list")
        .into_iter()
        .map(|ws| ws.name)
        .collect()
}

fn workspace_row(repo: &Repo, name: &str) -> WorkspaceInfo {
    repo.workspace_list()
        .expect("workspace list")
        .into_iter()
        .find(|ws| ws.name == name)
        .unwrap_or_else(|| panic!("{name} row"))
}

fn canonical(path: &std::path::Path) -> std::path::PathBuf {
    dunce::canonicalize(path).expect("canonical path")
}

fn recreate_elsewhere(repo_path: &std::path::Path, name: &str, dest: &std::path::Path) {
    run_jj_in(repo_path, &["workspace", "forget", name]);
    run_jj_in(
        repo_path,
        &[
            "workspace",
            "add",
            "--name",
            name,
            dest.to_str().expect("utf8 dest"),
        ],
    );
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
fn workspace_forget_rejects_the_current_workspace() {
    let temp_dir = init_jj_repo();
    let repo = Repo::open(&temp_dir.path().join("repo")).expect("open repo");

    let error = repo
        .workspace_forget("default", None)
        .expect_err("the current workspace cannot be forgotten");
    assert!(error.to_string().contains("current workspace"), "{error}");
}

#[test]
fn workspace_forget_supports_legacy_repositories_without_saved_roots() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    let dest = temp_dir.path().join("repo-feature");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("workspace add");
    SimpleWorkspaceStore::load(&repo_path.join(".jj").join("repo"))
        .expect("workspace store")
        .forget(&[WorkspaceName::new("feature")])
        .expect("remove saved root to simulate a legacy repository");

    let row = workspace_row(&repo, "feature");
    assert!(!row.is_path_resolved, "a legacy row has no root to act on");
    assert!(row.path.is_empty());

    repo.workspace_forget("feature", Some(dest.to_str().expect("utf8 dest")))
        .expect("the live checkout still proves ownership");
    assert_eq!(workspace_names(&repo), ["default"]);
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
        canonical(std::path::Path::new(&current.path)),
        canonical(&repo_path)
    );
    assert_eq!(
        current_op_id_ignoring_working_copy(&repo_path),
        op_before,
        "workspace enumeration must not create a snapshot operation"
    );
}

#[test]
fn workspace_list_sees_a_workspace_added_by_another_process() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let dest = temp_dir.path().join("feature-ws");
    run_jj_in(
        &repo_path,
        &[
            "workspace",
            "add",
            "--name",
            "feature",
            dest.to_str().expect("utf8 dest"),
        ],
    );

    assert_eq!(workspace_names(&repo), ["default", "feature"]);
}

#[test]
fn workspace_list_reports_sibling_working_copy_status() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let dest = temp_dir.path().join("feature-ws");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("add workspace");

    std::fs::write(dest.join("new-file"), "content\n").expect("write file in sibling");
    run_jj_in(&dest, &["describe", "-m", "sibling work\n\nbody"]);
    repo.refresh_working_copy()
        .expect("reload after sibling activity");

    let feature = workspace_row(&repo, "feature");
    assert!(!feature.is_current);
    assert!(feature.is_path_resolved);
    assert_eq!(feature.description, "sibling work");
    assert_eq!(feature.files_changed, 1);
    assert!(!feature.has_conflict);
    assert!(feature.timestamp > 0);
    assert!(feature.change_id.short_len > 0);
    assert_eq!(std::path::PathBuf::from(&feature.path), canonical(&dest));
}

#[test]
fn workspace_list_keeps_valid_rows_when_a_root_is_missing() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let dest = temp_dir.path().join("feature-ws");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("add workspace");
    std::fs::remove_dir_all(&dest).expect("remove workspace directory");

    let workspaces = repo.workspace_list().expect("list with a missing sibling");
    let current = workspaces
        .iter()
        .find(|ws| ws.is_current)
        .expect("current row");
    let feature = workspaces
        .iter()
        .find(|ws| ws.name == "feature")
        .expect("unresolved feature row");
    assert!(current.is_path_resolved);
    assert!(!feature.is_path_resolved);
    assert_eq!(
        std::path::PathBuf::from(&feature.path),
        canonical(temp_dir.path()).join("feature-ws")
    );

    let error = repo
        .workspace_forget("feature", Some(&feature.path))
        .expect_err("a missing root cannot be verified for deletion");
    assert!(error.to_string().contains("not a directory"), "{error}");
    repo.workspace_forget("feature", None)
        .expect("forget by name");
    assert_eq!(workspace_names(&repo), ["default"]);
}

#[test]
fn workspace_forget_with_root_rejects_a_checkout_that_is_not_that_workspace() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    let first = temp_dir.path().join("feature-first");
    repo.workspace_add(first.to_str().expect("utf8 dest"), "feature", "")
        .expect("add workspace");
    let first_root = first.to_str().expect("utf8 root");

    let aside = temp_dir.path().join("feature-aside");
    std::fs::rename(&first, &aside).expect("move the checkout aside");
    std::fs::create_dir(&first).expect("unrelated directory at the root");
    let error = repo
        .workspace_forget("feature", Some(first_root))
        .expect_err("an unrelated directory is not the workspace");
    assert!(error.to_string().contains("not a jj workspace"), "{error}");

    std::fs::remove_dir(&first).expect("remove the impostor");
    std::fs::rename(&aside, &first).expect("restore the checkout");
    recreate_elsewhere(
        &repo_path,
        "feature",
        &temp_dir.path().join("feature-second"),
    );
    let error = repo
        .workspace_forget("feature", Some(first_root))
        .expect_err("the old checkout still calls itself feature");
    assert!(error.to_string().contains("moved"), "{error}");

    repo.refresh_working_copy().expect("reload");
    assert!(workspace_names(&repo).contains(&"feature".to_owned()));
}

#[test]
fn sibling_workspace_working_copies_carry_their_name() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let dest = temp_dir.path().join("feature-ws");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("add workspace");

    let entries = repo.log("all()").expect("log");
    let named: Vec<&jayjay_core::ChangeInfo> = entries
        .iter()
        .filter(|c| c.workspaces == ["feature"])
        .collect();
    assert_eq!(
        named.len(),
        1,
        "exactly one commit carries the sibling name"
    );
    assert!(
        !named[0].is_working_copy,
        "the sibling working copy is not this workspace's @"
    );
    assert!(
        entries
            .iter()
            .all(|c| !c.is_working_copy || c.workspaces.is_empty()),
        "this workspace's @ must not repeat its own name"
    );
}
