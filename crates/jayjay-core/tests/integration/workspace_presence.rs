use jayjay_core::{Repo, WorkspacePresence};
use jj_test::{init_jj_repo, run_jj_in};

#[test]
fn workspace_presence_follows_forget_recreate_and_delete() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    let first = temp_dir.path().join("feature-first");
    repo.workspace_add(first.to_str().expect("utf8 dest"), "feature", "")
        .expect("add workspace");
    let old = Repo::open(&first).expect("open secondary");
    assert_eq!(old.workspace_presence(), WorkspacePresence::Exists);

    run_jj_in(&repo_path, &["workspace", "forget", "feature"]);
    assert!(
        first.join(".jj").exists(),
        "forget leaves the checkout behind"
    );
    assert_eq!(old.workspace_presence(), WorkspacePresence::Gone);
    assert_eq!(repo.workspace_presence(), WorkspacePresence::Exists);

    let second = temp_dir.path().join("feature-second");
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
    let new = Repo::open(&second).expect("open recreated workspace");
    assert_eq!(old.workspace_presence(), WorkspacePresence::Gone);
    assert_eq!(new.workspace_presence(), WorkspacePresence::Exists);

    run_jj_in(&repo_path, &["workspace", "forget", "feature"]);
    std::fs::remove_dir_all(&second).expect("delete checkout");
    assert_eq!(new.workspace_presence(), WorkspacePresence::Gone);
}

#[test]
fn unreadable_repo_leaves_presence_unknown() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    assert_eq!(repo.workspace_presence(), WorkspacePresence::Exists);

    std::fs::write(repo_path.join(".jj").join("working_copy").join("type"), "x")
        .expect("scramble working copy type");

    assert_eq!(repo.workspace_presence(), WorkspacePresence::Unknown);
}

#[test]
fn workspace_primary_root_resolves_secondaries_to_the_primary() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let dest = temp_dir.path().join("feature-ws");
    repo.workspace_add(dest.to_str().expect("utf8 dest"), "feature", "")
        .expect("add workspace");

    let canonical_repo = dunce::canonicalize(&repo_path).expect("canonical repo");
    let primary = jayjay_core::workspace_primary_root(repo_path.to_str().expect("utf8"))
        .expect("primary resolves");
    assert_eq!(std::path::PathBuf::from(primary), canonical_repo);
    let from_secondary = jayjay_core::workspace_primary_root(dest.to_str().expect("utf8"))
        .expect("secondary resolves");
    assert_eq!(std::path::PathBuf::from(from_secondary), canonical_repo);
    assert_eq!(
        jayjay_core::workspace_primary_root(temp_dir.path().to_str().expect("utf8")),
        None,
        "non-jj directories resolve to nothing"
    );
}
