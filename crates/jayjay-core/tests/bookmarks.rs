use std::fs;
use std::path::PathBuf;
use std::process::Command;

use jayjay_core::{ChangeInfo, Repo};
use jj_test::{
    LinearFixture, configure_test_user, init_colocated, init_jj_repo, run_command, run_git, run_jj,
    run_jj_in,
};

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
}

struct ConflictedFeatureFixture {
    _work_dir: tempfile::TempDir,
    alice: PathBuf,
}

fn conflicted_feature_fixture() -> ConflictedFeatureFixture {
    // Local move + fetch of a remote move of the same bookmark produces `name??` on both commits.
    let work_dir = tempfile::tempdir().expect("create work dir");
    let origin = work_dir.path().join("origin.git");
    let alice = work_dir.path().join("alice");
    let bob = work_dir.path().join("bob");
    let origin_str = origin.to_str().expect("utf-8");
    let bob_str = bob.to_str().expect("utf-8");

    run_command(
        "git",
        &["init".into(), "--bare".into(), origin_str.into()],
        Command::new("git").args(["init", "--bare", origin_str]),
    );

    init_colocated(&alice);
    configure_test_user(&alice);
    fs::write(alice.join("base.txt"), "base\n").expect("write base");
    run_jj_in(&alice, &["describe", "-m", "base"]);
    run_jj_in(&alice, &["bookmark", "create", "feature", "-r", "@"]);
    run_git(&alice, &["remote", "add", "origin", origin_str]);
    run_jj_in(
        &alice,
        &["git", "push", "--bookmark", "feature", "--remote", "origin"],
    );

    run_jj_in(&alice, &["new", "-m", "alice-move"]);
    fs::write(alice.join("alice.txt"), "alice\n").expect("write alice");
    run_jj_in(&alice, &["bookmark", "set", "feature", "-r", "@"]);

    run_jj(&["git", "clone", "--colocate", origin_str, bob_str]);
    configure_test_user(&bob);
    run_jj_in(&bob, &["bookmark", "track", "feature@origin"]);
    run_jj_in(&bob, &["new", "-m", "bob-move", "-r", "feature"]);
    fs::write(bob.join("bob.txt"), "bob\n").expect("write bob");
    run_jj_in(&bob, &["bookmark", "set", "feature", "-r", "@"]);
    run_jj_in(
        &bob,
        &["git", "push", "--bookmark", "feature", "--remote", "origin"],
    );
    run_jj_in(&alice, &["git", "fetch"]);

    ConflictedFeatureFixture {
        _work_dir: work_dir,
        alice,
    }
}

fn feature_targets(repo: &Repo) -> Vec<ChangeInfo> {
    repo.log("all()")
        .expect("load log")
        .into_iter()
        .filter(|change| change.bookmarks.iter().any(|name| name == "feature"))
        .collect()
}

fn listed_feature(repo: &Repo) -> jayjay_core::BookmarkInfo {
    repo.list_bookmarks()
        .expect("list bookmarks")
        .into_iter()
        .find(|bookmark| bookmark.name == "feature")
        .expect("feature bookmark")
}

#[test]
fn remove_bookmark_from_rev_keeps_the_other_conflicted_target() {
    let fixture = conflicted_feature_fixture();
    let repo = Repo::open(&fixture.alice).expect("open alice");
    assert!(
        listed_feature(&repo).is_conflicted,
        "fetched remote move should conflict with alice's local move"
    );
    let feature_changes = feature_targets(&repo);
    assert_eq!(
        feature_changes.len(),
        2,
        "conflicted bookmark must appear on both target commits"
    );
    let alice_move = feature_changes
        .into_iter()
        .find(|change| change.description.trim() == "alice-move")
        .expect("alice-move target");

    repo.remove_bookmark_from_rev("feature", &alice_move.commit_id.id)
        .expect("remove feature from alice-move");

    let remaining = feature_targets(&repo);
    assert_eq!(
        remaining
            .iter()
            .map(|change| change.description.trim())
            .collect::<Vec<_>>(),
        vec!["bob-move"],
        "removing the chip from alice-move should leave feature only on bob-move"
    );
    assert!(
        !listed_feature(&repo).is_conflicted,
        "a single remaining target should resolve the conflict"
    );
}

#[test]
fn remove_bookmark_from_rev_on_a_resolved_bookmark() {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["bookmark", "create", "side", "-r", "@--"]);
    let repo = Repo::open(&fixture.path).expect("open repo");

    let err = repo
        .remove_bookmark_from_rev("side", "@")
        .expect_err("working copy does not carry side");
    assert!(
        err.to_string().contains("does not point at this change"),
        "unexpected error: {err}"
    );

    let target = repo
        .log("all()")
        .expect("load log")
        .into_iter()
        .find(|change| change.bookmarks.iter().any(|name| name == "side"))
        .expect("side bookmark");
    repo.remove_bookmark_from_rev("side", &target.commit_id.id)
        .expect("remove only target");

    let leftover = repo
        .log("all()")
        .expect("reload log")
        .into_iter()
        .any(|change| change.bookmarks.iter().any(|name| name == "side"));
    assert!(
        !leftover,
        "removing the only target should delete the bookmark"
    );
}

#[test]
fn a_bookmark_known_only_to_the_backing_git_repo_is_local_only() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    run_jj_in(&repo_path, &["bookmark", "create", "local-only", "-r", "@"]);
    run_jj_in(&repo_path, &["st"]);

    let bookmark = Repo::open(&repo_path)
        .expect("open repo")
        .list_bookmarks()
        .expect("list bookmarks")
        .into_iter()
        .find(|bookmark| bookmark.name == "local-only")
        .expect("local-only bookmark");

    assert!(!bookmark.is_tracking_remote);
    assert!(bookmark.tracked_remotes.is_empty());
    assert!(
        bookmark.available_remotes.is_empty(),
        "{:?}",
        bookmark.available_remotes
    );
}
