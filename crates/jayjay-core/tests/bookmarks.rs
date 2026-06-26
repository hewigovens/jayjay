use std::fs;
use std::process::Command;

use jayjay_core::Repo;
use jj_test::{run_command, run_git, run_jj};

#[test]
fn list_bookmarks_includes_untracked_remote_branch() {
    // Alice pushes a branch to origin; bob clones and sees it as `name@origin` with no local target.
    let work_dir = tempfile::tempdir().expect("create work dir");
    let bare_path = work_dir.path().join("origin.git");
    let alice_path = work_dir.path().join("alice");
    let bob_path = work_dir.path().join("bob");

    let bare_str = bare_path.to_str().expect("bare path utf-8");
    let alice_str = alice_path.to_str().expect("alice path utf-8");
    let bob_str = bob_path.to_str().expect("bob path utf-8");

    // Bare origin
    run_command(
        "git",
        &["init".into(), "--bare".into(), bare_str.into()],
        Command::new("git").args(["init", "--bare", bare_str]),
    );

    // Alice: colocated jj+git repo, two commits, push main and feature
    run_jj(&["git", "init", "--colocate", alice_str]);
    run_jj(&[
        "-R",
        alice_str,
        "config",
        "set",
        "--repo",
        "user.name",
        "Alice",
    ]);
    run_jj(&[
        "-R",
        alice_str,
        "config",
        "set",
        "--repo",
        "user.email",
        "alice@example.com",
    ]);
    fs::write(alice_path.join("a.txt"), "alice initial\n").expect("write a.txt");
    run_jj(&["-R", alice_str, "describe", "-m", "initial"]);
    run_jj(&["-R", alice_str, "bookmark", "create", "main", "-r", "@"]);
    run_jj(&["-R", alice_str, "new", "-m", "alice feature work"]);
    fs::write(alice_path.join("b.txt"), "alice feature\n").expect("write b.txt");
    run_jj(&[
        "-R",
        alice_str,
        "bookmark",
        "create",
        "alice-feature",
        "-r",
        "@",
    ]);
    run_git(&alice_path, &["remote", "add", "origin", bare_str]);
    run_jj(&[
        "-R",
        alice_str,
        "git",
        "push",
        "--bookmark",
        "main",
        "--bookmark",
        "alice-feature",
        "--remote",
        "origin",
    ]);

    // Bob: clone via jj; alice-feature should arrive as an untracked remote bookmark
    run_jj(&["git", "clone", "--colocate", bare_str, bob_str]);

    let repo = Repo::open(&bob_path).expect("open bob's repo");
    let bookmarks = repo.list_bookmarks().expect("list bookmarks");

    let orphan = bookmarks
        .iter()
        .find(|b| b.name == "alice-feature")
        .expect("alice-feature should appear in list_bookmarks");
    assert!(
        !orphan.has_local_target,
        "alice-feature has no local target in bob's repo"
    );
    assert!(
        !orphan.is_tracking_remote,
        "alice-feature is not tracked in bob's repo"
    );
    assert_eq!(
        orphan.available_remotes,
        vec!["origin".to_string()],
        "alice-feature is available only on origin"
    );
    assert!(
        !orphan.change_id.is_empty(),
        "orphan entry should carry the change id from the remote target"
    );

    let graph = repo.log_graph("alice-feature@origin").expect("log graph");
    let remote_change = graph
        .iter()
        .find(|entry| {
            entry
                .change
                .remote_bookmarks
                .contains(&"alice-feature@origin".to_string())
        })
        .expect("remote feature change should expose a remote bookmark label");
    assert_eq!(
        remote_change.change.remote_bookmarks,
        vec!["alice-feature@origin".to_string()],
        "DAG rows should expose remote-only bookmark labels"
    );
}
