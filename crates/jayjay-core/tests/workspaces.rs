//! Workspace add/list/forget behavior over jj, including operand safety for option-shaped destinations and workspace names.

use jayjay_core::{Repo, WorkspacePresence};
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

fn workspace_path(repo: &Repo, name: &str) -> std::path::PathBuf {
    let workspaces = repo.workspace_list().expect("workspace list");
    let workspace = workspaces
        .iter()
        .find(|ws| ws.name == name)
        .unwrap_or_else(|| panic!("{name} row"));
    std::fs::canonicalize(&workspace.path).expect("canonical workspace path")
}

fn forget_workspace(repo: &Repo, name: &str, path: &std::path::Path) {
    let path = std::fs::canonicalize(path).expect("canonical workspace path");
    let path = path.to_str().expect("utf8 workspace path");
    let expected_operation = repo
        .workspace_list()
        .expect("workspace list")
        .into_iter()
        .find(|workspace| workspace.name == name)
        .expect("workspace row")
        .operation_id;
    let operation = repo
        .workspace_removal_guard(name, path, &expected_operation)
        .expect("workspace removal guard");
    let warning = repo
        .workspace_forget(name, path, &operation)
        .expect("workspace forget");
    assert!(warning.is_none(), "{warning:?}");
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

    forget_workspace(&repo, "feature", &dest);
    let names = workspace_names(&repo);
    assert!(!names.contains(&"feature".to_owned()), "{names:?}");
}

#[test]
fn workspace_forget_supports_legacy_repositories_without_saved_roots() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    let dest = temp_dir.path().join("repo-feature");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("workspace add");
    let row = repo
        .workspace_list()
        .expect("workspace list")
        .into_iter()
        .find(|workspace| workspace.name == "feature")
        .expect("workspace row");
    SimpleWorkspaceStore::load(&repo_path.join(".jj").join("repo"))
        .expect("workspace store")
        .forget(&[WorkspaceName::new("feature")])
        .expect("remove saved root to simulate a legacy repository");

    let operation = repo
        .workspace_removal_guard("feature", &row.path, &row.operation_id)
        .expect("legacy workspace removal guard");
    let warning = repo
        .workspace_forget("feature", &row.path, &operation)
        .expect("legacy workspace forget");

    assert!(warning.is_none(), "{warning:?}");
    repo.refresh_working_copy().expect("reload after forget");
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

    forget_workspace(&repo, "hostile", &repo_path.join("-hostile"));
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

    let workspaces = repo.workspace_list().expect("workspace list");
    let feature = workspaces
        .iter()
        .find(|ws| ws.name == "feature")
        .expect("feature row");
    assert!(!feature.is_current);
    assert_eq!(feature.description, "sibling work");
    assert_eq!(feature.files_changed, 1);
    assert!(!feature.has_conflict);
    assert!(feature.timestamp > 0);
    assert!(feature.change_id.short_len > 0);
    assert_eq!(
        std::fs::canonicalize(&feature.path).expect("canonical sibling path"),
        std::fs::canonicalize(&dest).expect("canonical dest")
    );
}

/// One missing checkout must not hide valid siblings; the unresolved row remains non-actionable except for operation-locked Forget recovery.
#[test]
fn workspace_list_preserves_valid_rows_and_forgets_an_unresolved_root() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let dest = temp_dir.path().join("feature-ws");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("add workspace");
    let feature = repo
        .workspace_list()
        .expect("prime resolved workspace path cache")
        .into_iter()
        .find(|workspace| workspace.name == "feature")
        .expect("resolved feature row");
    assert!(feature.is_path_resolved);
    std::fs::remove_dir_all(&dest).expect("remove workspace directory");

    let workspaces = repo.workspace_list().expect("list with unresolved sibling");
    let current = workspaces
        .iter()
        .find(|workspace| workspace.is_current)
        .expect("current row");
    let feature = workspaces
        .iter()
        .find(|workspace| workspace.name == "feature")
        .expect("unresolved feature row");
    assert!(current.is_path_resolved);
    assert!(!feature.is_path_resolved);
    let canonical_temp = std::fs::canonicalize(temp_dir.path()).expect("canonical temp root");
    assert_eq!(
        std::path::PathBuf::from(&feature.path),
        dunce::simplified(&canonical_temp).join("feature-ws")
    );

    let warning = repo
        .workspace_forget_unresolved("feature", &feature.operation_id)
        .expect("forget unresolved workspace");
    assert!(warning.is_none(), "{warning:?}");
    repo.refresh_working_copy().expect("reload after forget");
    assert_eq!(workspace_names(&repo), ["default"]);
}

#[test]
fn unresolved_workspace_forget_rejects_a_root_that_became_available() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    let dest = temp_dir.path().join("feature-ws");
    let held = temp_dir.path().join("feature-held");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("add workspace");
    std::fs::rename(&dest, &held).expect("hide workspace directory");
    let feature = repo
        .workspace_list()
        .expect("list unresolved workspace")
        .into_iter()
        .find(|workspace| workspace.name == "feature")
        .expect("feature row");
    assert!(!feature.is_path_resolved);
    std::fs::rename(&held, &dest).expect("restore workspace directory");

    let error = repo
        .workspace_forget_unresolved("feature", &feature.operation_id)
        .expect_err("a recovered root must require normal guarded removal");

    assert!(
        error.to_string().contains("root became available"),
        "{error}"
    );
    assert!(workspace_names(&repo).contains(&"feature".to_owned()));
}

#[test]
fn workspace_list_retries_from_an_external_operation_generation() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo before external operation");
    let dest = temp_dir.path().join("feature-ws");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("add workspace");

    run_jj_in(&repo_path, &["describe", "-m", "externally updated"]);

    let workspaces = repo
        .workspace_list()
        .expect("retry workspace list from the newer operation");
    let current = workspaces
        .iter()
        .find(|workspace| workspace.is_current)
        .expect("current workspace row");
    assert_eq!(current.description, "externally updated");
    assert!(
        workspaces
            .iter()
            .any(|workspace| workspace.name == "feature")
    );
}

/// Another process can forget a name and recreate it at a different root without the name ever leaving the listing, so cached roots must not survive the operations that moved it.
#[test]
fn workspace_list_reresolves_a_name_recreated_at_another_root() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let first = temp_dir.path().join("feature-first");
    repo.workspace_add(first.to_str().expect("utf8 dest"), "feature", "")
        .expect("add workspace");
    assert_eq!(
        workspace_path(&repo, "feature"),
        std::fs::canonicalize(&first).expect("canonical first")
    );

    let second = temp_dir.path().join("feature-second");
    run_jj_in(&repo_path, &["workspace", "forget", "feature"]);
    run_jj_in(
        &repo_path,
        &[
            "workspace",
            "add",
            "--name",
            "feature",
            second.to_str().expect("utf8 dest"),
        ],
    );
    repo.refresh_working_copy()
        .expect("reload after external recreate");

    assert_eq!(
        workspace_path(&repo, "feature"),
        std::fs::canonicalize(&second).expect("canonical second"),
        "the recreated root must replace the cached one"
    );
}

#[test]
fn stale_workspace_removal_guard_does_not_forget_a_recreated_name() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let first = temp_dir.path().join("feature-first");
    repo.workspace_add(first.to_str().expect("utf8 dest"), "feature", "")
        .expect("add workspace");
    let first = std::fs::canonicalize(first).expect("canonical first root");
    let expected_operation = repo
        .workspace_list()
        .expect("workspace list")
        .into_iter()
        .find(|workspace| workspace.name == "feature")
        .expect("workspace row")
        .operation_id;
    let operation = repo
        .workspace_removal_guard(
            "feature",
            first.to_str().expect("utf8 root"),
            &expected_operation,
        )
        .expect("workspace removal guard");

    let second = temp_dir.path().join("feature-second");
    run_jj_in(&repo_path, &["workspace", "forget", "feature"]);
    run_jj_in(
        &repo_path,
        &[
            "workspace",
            "add",
            "--name",
            "feature",
            second.to_str().expect("utf8 dest"),
        ],
    );

    let error = repo
        .workspace_forget("feature", first.to_str().expect("utf8 root"), &operation)
        .expect_err("a stale removal guard must not forget the recreated workspace");
    assert!(
        error.to_string().contains("changed after confirmation"),
        "{error}"
    );
    repo.refresh_working_copy()
        .expect("reload after external recreate");
    assert_eq!(
        workspace_path(&repo, "feature"),
        std::fs::canonicalize(second).expect("canonical second root")
    );
}

#[test]
fn stale_workspace_row_does_not_validate_a_recreated_name_at_the_same_root() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let dest = temp_dir.path().join("feature-ws");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("add workspace");
    let listed_operation = repo
        .workspace_list()
        .expect("workspace list")
        .into_iter()
        .find(|workspace| workspace.name == "feature")
        .expect("workspace row")
        .operation_id;

    run_jj_in(&repo_path, &["workspace", "forget", "feature"]);
    std::fs::remove_dir_all(&dest).expect("remove forgotten checkout");
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

    let error = repo
        .workspace_removal_guard(
            "feature",
            dest.to_str().expect("utf8 dest"),
            &listed_operation,
        )
        .expect_err("a stale row must not validate its same-name, same-root replacement");
    assert!(
        error.to_string().contains("changed after it was listed"),
        "{error}"
    );
}

#[test]
fn workspace_removal_guard_rejects_a_replaced_checkout_directory() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let dest = temp_dir.path().join("feature-ws");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("add workspace");
    let expected_operation = repo
        .workspace_list()
        .expect("workspace list")
        .into_iter()
        .find(|workspace| workspace.name == "feature")
        .expect("workspace row")
        .operation_id;
    std::fs::rename(&dest, temp_dir.path().join("feature-original"))
        .expect("move original workspace");
    std::fs::create_dir(&dest).expect("create unrelated replacement");

    let error = repo
        .workspace_removal_guard(
            "feature",
            dest.to_str().expect("utf8 dest"),
            &expected_operation,
        )
        .expect_err("an unrelated replacement must not receive a removal guard");
    assert!(
        error.to_string().contains("prepare workspace removal"),
        "{error}"
    );
}

/// A workspace forgotten by another process keeps its checkout directory, `.jj` and all, so presence has to come from the view rather than the filesystem.
#[test]
fn forgotten_workspace_is_gone_while_its_directory_remains() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let dest = temp_dir.path().join("feature-ws");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("add workspace");
    let secondary = Repo::open(&dest).expect("open secondary");
    assert_eq!(secondary.workspace_presence(), WorkspacePresence::Exists);

    run_jj_in(&repo_path, &["workspace", "forget", "feature"]);

    assert!(
        dest.join(".jj").exists(),
        "forget leaves the checkout behind"
    );
    assert_eq!(
        secondary.workspace_presence(),
        WorkspacePresence::Gone,
        "a forgotten workspace must not report itself as existing"
    );
    assert_eq!(
        repo.workspace_presence(),
        WorkspacePresence::Exists,
        "the primary is untouched"
    );
}

/// The recreated workspace keeps the name in the view, so only the recorded root tells the old checkout that the name is no longer its own.
#[test]
fn workspace_recreated_at_another_root_leaves_the_old_checkout_gone() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let first = temp_dir.path().join("feature-first");
    repo.workspace_add(first.to_str().expect("utf8 dest"), "feature", "")
        .expect("add workspace");
    let secondary = Repo::open(&first).expect("open secondary");
    assert_eq!(secondary.workspace_presence(), WorkspacePresence::Exists);

    let second = temp_dir.path().join("feature-second");
    run_jj_in(&repo_path, &["workspace", "forget", "feature"]);
    run_jj_in(
        &repo_path,
        &[
            "workspace",
            "add",
            "--name",
            "feature",
            second.to_str().expect("utf8 dest"),
        ],
    );

    assert_eq!(
        secondary.workspace_presence(),
        WorkspacePresence::Gone,
        "the name now belongs to another checkout"
    );
    assert_eq!(
        Repo::open(&second)
            .expect("open recreated workspace")
            .workspace_presence(),
        WorkspacePresence::Exists,
        "the checkout that owns the name is unaffected"
    );
}

/// Forget & Delete from Disk removes the checkout outright, and the refresh failure it triggers must still close the window.
#[test]
fn deleted_workspace_directory_is_gone() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let dest = temp_dir.path().join("feature-ws");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("add workspace");
    let secondary = Repo::open(&dest).expect("open secondary");

    run_jj_in(&repo_path, &["workspace", "forget", "feature"]);
    std::fs::remove_dir_all(&dest).expect("delete checkout");

    assert_eq!(secondary.workspace_presence(), WorkspacePresence::Gone);
}

/// An unreadable repo is not proof of removal: the window must keep showing the real refresh error instead of closing.
#[test]
fn unreadable_repo_leaves_presence_unknown() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    assert_eq!(repo.workspace_presence(), WorkspacePresence::Exists);

    std::fs::write(repo_path.join(".jj").join("working_copy").join("type"), "x")
        .expect("scramble working copy type");

    assert_eq!(
        repo.workspace_presence(),
        WorkspacePresence::Unknown,
        "a load failure with the checkout still on disk stays undecided"
    );
}

#[test]
fn workspace_primary_root_resolves_secondaries_to_the_primary() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let dest = temp_dir.path().join("feature-ws");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("add workspace");

    let canonical_repo = std::fs::canonicalize(&repo_path).expect("canonical repo");
    let primary = jayjay_core::workspace_primary_root(repo_path.to_str().expect("utf8"))
        .expect("primary resolves");
    assert_eq!(
        std::fs::canonicalize(&primary).expect("canonical primary"),
        canonical_repo
    );
    let from_secondary = jayjay_core::workspace_primary_root(dest.to_str().expect("utf8"))
        .expect("secondary resolves");
    assert_eq!(std::path::PathBuf::from(from_secondary), canonical_repo);
    assert_eq!(
        jayjay_core::workspace_primary_root(temp_dir.path().to_str().expect("utf8")),
        None,
        "non-jj directories resolve to nothing"
    );
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
