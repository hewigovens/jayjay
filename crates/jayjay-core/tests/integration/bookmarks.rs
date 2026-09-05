use std::fs;
use std::path::PathBuf;
use std::process::Command;

use jayjay_core::{ChangeInfo, Repo};
use jj_test::{
    LinearFixture, configure_test_user, init_colocated, init_jj_repo, run_command, run_git, run_jj,
    run_jj_in,
};

#[test]
fn sync_only_tracks_explicitly_requested_bookmarks() {
    let work_dir = tempfile::tempdir().expect("create work dir");
    let bare_path = work_dir.path().join("origin.git");
    let alice_path = work_dir.path().join("alice");
    let bob_path = work_dir.path().join("bob");

    let bare_str = bare_path.to_str().expect("bare path utf-8");
    let bob_str = bob_path.to_str().expect("bob path utf-8");

    run_command(
        "git",
        &["init".into(), "--bare".into(), bare_str.into()],
        Command::new("git").args(["init", "--bare", bare_str]),
    );

    init_colocated(&alice_path);
    configure_test_user(&alice_path);
    fs::write(alice_path.join("a.txt"), "alice initial\n").expect("write a.txt");
    run_jj_in(&alice_path, &["describe", "-m", "initial"]);
    run_jj_in(&alice_path, &["bookmark", "create", "main", "-r", "@"]);
    run_jj_in(&alice_path, &["new", "-m", "alice feature work"]);
    fs::write(alice_path.join("b.txt"), "alice feature\n").expect("write b.txt");
    run_jj_in(
        &alice_path,
        &["bookmark", "create", "alice-feature", "-r", "@"],
    );
    run_git(&alice_path, &["remote", "add", "origin", bare_str]);
    run_jj_in(
        &alice_path,
        &[
            "git",
            "push",
            "--bookmark",
            "main",
            "--bookmark",
            "alice-feature",
            "--remote",
            "origin",
        ],
    );

    run_jj(&["git", "clone", "--colocate", bare_str, bob_str]);
    configure_test_user(&bob_path);

    let repo = Repo::open(&bob_path).expect("open bob's repo");
    let bookmarks = repo.list_bookmarks().expect("list bookmarks");

    let orphan = bookmarks
        .iter()
        .find(|b| b.name == "alice-feature")
        .expect("alice-feature should appear in list_bookmarks");
    assert!(!orphan.has_local_target);
    assert!(!orphan.is_tracking_remote);
    assert_eq!(orphan.available_remotes, ["origin"]);
    assert!(!orphan.change_id.is_empty());

    repo.track_bookmark("main", "origin").expect("track main");
    run_jj_in(
        &alice_path,
        &["new", "-r", "alice-feature", "-m", "updated feature"],
    );
    run_jj_in(
        &alice_path,
        &["bookmark", "set", "alice-feature", "-r", "@"],
    );
    run_jj_in(&alice_path, &["git", "push", "--bookmark", "alice-feature"]);

    repo.git_fetch("origin", &repo.sync_token()).expect("pull");
    let after = repo.list_bookmarks().expect("bookmarks after pull");
    let remote = after
        .iter()
        .find(|b| b.name == "alice-feature")
        .expect("remote bookmark");
    assert!(!remote.has_local_target);
    assert!(!remote.is_tracking_remote);
    assert_ne!(remote.change_id, orphan.change_id);
    assert!(
        repo.log("mutable() & ::remote_bookmarks(exact:\"alice-feature\", exact:\"origin\")")
            .unwrap()
            .is_empty()
    );

    run_jj_in(&bob_path, &["new", "-r", "main", "-m", "bob main update"]);
    fs::write(bob_path.join("bob.txt"), "bob update\n").expect("write bob update");
    run_jj_in(&bob_path, &["bookmark", "set", "main", "-r", "@"]);
    repo.git_push("", &repo.sync_token())
        .expect("ordinary push");
    assert_eq!(
        run_git(&bare_path, &["rev-parse", "refs/heads/main"]).stdout,
        format!("{}\n", repo.log("main").unwrap()[0].commit_id.id).into_bytes()
    );
    assert!(
        repo.list_bookmarks()
            .unwrap()
            .iter()
            .any(|b| b.name == "alice-feature" && !b.has_local_target)
    );

    run_jj_in(&bob_path, &["new", "-r", "main", "-m", "bob feature work"]);
    run_jj_in(&bob_path, &["bookmark", "create", "bob-feature", "-r", "@"]);
    let message = repo.git_push("", &repo.sync_token()).expect("default push");
    assert!(
        message.contains("bob-feature") && message.contains("track"),
        "{message}"
    );
    repo.git_push("bob-feature", &repo.sync_token())
        .expect("explicit bookmark push");
    let message = repo
        .git_push("bob-feature", &repo.sync_token())
        .expect("up-to-date push");
    assert!(
        message.contains("Nothing changed") && !message.contains("create a bookmark"),
        "{message}"
    );
    let mut tracked: Vec<_> = repo
        .list_bookmarks()
        .unwrap()
        .into_iter()
        .filter(|b| b.is_tracking_remote)
        .map(|b| b.name)
        .collect();
    tracked.sort();
    assert_eq!(tracked, ["bob-feature", "main"]);

    repo.git_pull_bookmark("alice-feature", &repo.sync_token())
        .expect("explicit bookmark pull");
    assert!(
        repo.list_bookmarks()
            .unwrap()
            .iter()
            .any(|b| b.name == "alice-feature" && b.has_local_target && b.is_tracking_remote)
    );
}

#[test]
fn deleted_bookmark_preserves_tracking_per_remote() {
    let fixture = LinearFixture::build();
    for (remote, target) in [("origin", "HEAD"), ("upstream", "HEAD~1")] {
        run_git(
            &fixture.path,
            &[
                "remote",
                "add",
                remote,
                &format!("https://example.invalid/{remote}.git"),
            ],
        );
        run_git(
            &fixture.path,
            &[
                "update-ref",
                &format!("refs/remotes/{remote}/feature"),
                target,
            ],
        );
    }
    run_jj_in(&fixture.path, &["status"]);
    let repo = Repo::open(&fixture.path).expect("open repo");
    repo.track_bookmark("feature", "origin")
        .expect("track origin bookmark");
    repo.delete_bookmark("feature")
        .expect("delete local bookmark");
    let deleted = listed_feature(&repo);
    assert!(deleted.is_deleted && !deleted.has_local_target);
    assert_eq!(deleted.available_remotes, ["origin", "upstream"]);
    assert_eq!(deleted.tracked_remotes, ["origin"]);
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
fn rename_bookmark_rejects_a_live_destination() {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["bookmark", "create", "source", "-r", "@"]);
    let repo = Repo::open(&fixture.path).expect("open repo");

    let err = repo
        .rename_bookmark("source", "main")
        .expect_err("main already exists");
    assert!(
        err.to_string().contains("already exists"),
        "unexpected error: {err}"
    );

    let names: Vec<_> = repo
        .list_bookmarks()
        .expect("list bookmarks")
        .into_iter()
        .filter(|bookmark| !bookmark.is_deleted)
        .map(|bookmark| bookmark.name)
        .collect();
    assert!(names.iter().any(|name| name == "source"));
    assert!(names.iter().any(|name| name == "main"));
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
