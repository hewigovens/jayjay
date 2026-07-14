use std::fs;
use std::path::PathBuf;
use std::process::Command;

use jayjay_core::{Repo, SubmitStackLayer};
use jj_test::{init_jj_repo, run_command, run_git, run_jj};

fn submit_layer(change_id: String, bookmark: &str, title: &str) -> SubmitStackLayer {
    SubmitStackLayer {
        change_id,
        bookmark: bookmark.to_owned(),
        title: title.to_owned(),
        body: String::new(),
    }
}

#[test]
fn detect_stack_builds_linear_layers_with_dependent_bases() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    // base → layer one → layer two(@)
    run_jj(&["-R", repo_str, "describe", "-m", "base change"]);
    run_jj(&["-R", repo_str, "bookmark", "create", "base", "-r", "@"]);
    run_jj(&["-R", repo_str, "new", "-m", "layer one"]);
    run_jj(&["-R", repo_str, "new", "-m", "layer two"]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let stack = repo.detect_stack("base", "@").expect("detect stack");

    assert_eq!(stack.layers.len(), 2);
    assert_eq!(stack.layers[0].title, "layer one");
    assert_eq!(stack.layers[1].title, "layer two");

    // Bottom PR targets the trunk branch; the upper PR targets the layer below.
    assert_eq!(stack.layers[0].base, stack.base_bookmark);
    assert_eq!(stack.layers[1].base, stack.layers[0].bookmark);

    // Bookmarks are auto-assigned (no existing ones) and slugged from the title.
    assert!(!stack.layers[0].bookmark_existed);
    assert!(
        stack.layers[0].bookmark.starts_with("layer-one-"),
        "got {}",
        stack.layers[0].bookmark
    );
}

#[test]
fn detect_stack_reuses_an_existing_bookmark() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    run_jj(&["-R", repo_str, "bookmark", "create", "base", "-r", "@"]);
    run_jj(&["-R", repo_str, "new", "-m", "feature"]);
    run_jj(&[
        "-R",
        repo_str,
        "bookmark",
        "create",
        "my-feature",
        "-r",
        "@",
    ]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let stack = repo.detect_stack("base", "@").expect("detect");

    assert_eq!(stack.layers.len(), 1);
    assert!(stack.layers[0].bookmark_existed);
    assert_eq!(stack.layers[0].bookmark, "my-feature");
}

#[test]
fn submit_stack_revalidates_order_before_moving_bookmarks() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    run_jj(&["-R", repo_str, "describe", "-m", "base"]);
    run_jj(&["-R", repo_str, "bookmark", "create", "main", "-r", "@"]);
    run_jj(&["-R", repo_str, "new", "-m", "layer one"]);
    run_jj(&["-R", repo_str, "new", "-m", "layer two"]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let stack = repo.detect_stack("main", "@").expect("detect stack");
    let top_change = stack.layers[1].change_id.clone();
    let submitted = stack
        .layers
        .iter()
        .map(|layer| submit_layer(layer.change_id.clone(), &layer.bookmark, &layer.title))
        .collect();

    run_jj(&["-R", repo_str, "rebase", "-r", &top_change, "-d", "main"]);

    let error = repo
        .submit_stack(submitted)
        .expect_err("reparented stack must fail");
    assert!(
        error.to_string().contains("stack must be linear"),
        "unexpected error: {error}"
    );
    assert_eq!(
        repo.list_bookmarks()
            .expect("list bookmarks after failure")
            .iter()
            .filter(|bookmark| bookmark.has_local_target && !bookmark.is_deleted)
            .map(|bookmark| bookmark.name.as_str())
            .collect::<Vec<_>>(),
        ["main"],
        "submission must not move or create bookmarks"
    );
}

#[test]
fn submit_stack_rejects_bookmark_owned_by_another_change_before_mutation() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    run_jj(&["-R", repo_str, "describe", "-m", "base"]);
    run_jj(&["-R", repo_str, "bookmark", "create", "main", "-r", "@"]);
    run_jj(&["-R", repo_str, "new", "-m", "feature"]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let feature = repo.show_summary("@").expect("show feature").info;
    let main_before = repo
        .list_bookmarks()
        .expect("list bookmarks")
        .into_iter()
        .find(|bookmark| bookmark.name == "main")
        .expect("main bookmark")
        .change_id
        .id;

    let error = repo
        .submit_stack(vec![submit_layer(feature.change_id.id, "main", "Feature")])
        .expect_err("bookmark collision must fail");

    assert!(
        error
            .to_string()
            .contains("Bookmark \"main\" already belongs to change"),
        "unexpected error: {error}"
    );
    let bookmarks = repo.list_bookmarks().expect("list bookmarks after failure");
    assert_eq!(
        bookmarks
            .iter()
            .find(|bookmark| bookmark.name == "main")
            .expect("main bookmark preserved")
            .change_id
            .id,
        main_before
    );
    assert_eq!(
        bookmarks
            .iter()
            .filter(|bookmark| bookmark.has_local_target && !bookmark.is_deleted)
            .count(),
        1,
        "submission must not create another local bookmark"
    );
}

#[test]
fn submit_stack_rejects_remote_only_bookmark_before_mutation() {
    let (_work_dir, bob_path) = remote_bookmark_fixture();
    let repo = Repo::open(&bob_path).expect("open bob repo");
    let feature = repo.show_summary("@").expect("show bob feature").info;

    let error = repo
        .submit_stack(vec![submit_layer(
            feature.change_id.id,
            "reserved",
            "Bob feature",
        )])
        .expect_err("remote-only bookmark collision must fail");

    assert!(
        error
            .to_string()
            .contains("Bookmark \"reserved\" already belongs to change"),
        "unexpected error: {error}"
    );
    let reserved = repo
        .list_bookmarks()
        .expect("list bookmarks after failure")
        .into_iter()
        .find(|bookmark| bookmark.name == "reserved")
        .expect("reserved remote bookmark preserved");
    assert!(
        !reserved.has_local_target,
        "submission must not create a local bookmark"
    );
}

#[test]
fn submit_stack_rejects_diverged_untracked_origin_bookmark() {
    let (_work_dir, bob_path) = remote_bookmark_fixture();
    let bob_str = bob_path.to_str().expect("bob path utf-8");
    run_jj(&["-R", bob_str, "bookmark", "create", "reserved", "-r", "@"]);
    run_jj(&["-R", bob_str, "bookmark", "untrack", "reserved@origin"]);
    let repo = Repo::open(&bob_path).expect("open bob repo");
    let feature = repo.show_summary("@").expect("show bob feature").info;
    let local_before = repo
        .list_bookmarks()
        .expect("list bookmarks")
        .into_iter()
        .find(|bookmark| bookmark.name == "reserved")
        .expect("local reserved bookmark");
    assert_eq!(local_before.change_id.id, feature.change_id.id);
    assert!(
        !local_before
            .tracked_remotes
            .iter()
            .any(|remote| remote == "origin"),
        "fixture must keep reserved@origin untracked"
    );

    let error = repo
        .submit_stack(vec![submit_layer(
            feature.change_id.id.clone(),
            "reserved",
            "Bob feature",
        )])
        .expect_err("diverged origin bookmark collision must fail");

    assert!(
        error
            .to_string()
            .contains("Bookmark \"reserved\" already belongs to change"),
        "unexpected error: {error}"
    );
    let local_after = repo
        .list_bookmarks()
        .expect("list bookmarks after failure")
        .into_iter()
        .find(|bookmark| bookmark.name == "reserved")
        .expect("local reserved bookmark preserved");
    assert_eq!(local_after.change_id.id, feature.change_id.id);
}

fn remote_bookmark_fixture() -> (tempfile::TempDir, PathBuf) {
    let work_dir = tempfile::tempdir().expect("create work dir");
    let bare_path = work_dir.path().join("origin.git");
    let alice_path = work_dir.path().join("alice");
    let bob_path = work_dir.path().join("bob");
    let bare_str = bare_path.to_str().expect("bare path utf-8");
    let alice_str = alice_path.to_str().expect("alice path utf-8");
    let bob_str = bob_path.to_str().expect("bob path utf-8");

    run_command(
        "git",
        &[
            "init".into(),
            "--bare".into(),
            "--initial-branch=main".into(),
            bare_str.into(),
        ],
        Command::new("git").args(["init", "--bare", "--initial-branch=main", bare_str]),
    );
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
    fs::write(alice_path.join("main.txt"), "main\n").expect("write main");
    run_jj(&["-R", alice_str, "describe", "-m", "main"]);
    run_jj(&["-R", alice_str, "bookmark", "create", "main", "-r", "@"]);
    run_jj(&["-R", alice_str, "new", "-m", "reserved remote feature"]);
    fs::write(alice_path.join("reserved.txt"), "reserved\n").expect("write feature");
    run_jj(&["-R", alice_str, "bookmark", "create", "reserved", "-r", "@"]);
    run_git(&alice_path, &["remote", "add", "origin", bare_str]);
    run_jj(&[
        "-R",
        alice_str,
        "git",
        "push",
        "--bookmark",
        "main",
        "--bookmark",
        "reserved",
        "--remote",
        "origin",
    ]);

    run_jj(&["git", "clone", "--colocate", bare_str, bob_str]);
    run_jj(&["-R", bob_str, "new", "main", "-m", "bob feature"]);
    (work_dir, bob_path)
}
