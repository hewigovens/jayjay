#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::thread;
use std::time::{Duration, Instant};

use jayjay_core::{JayError, Repo};
use jj_test::{find_bookmark, git_stdout, init_jj_repo, repo_with_remote, run_git, run_jj_in};

#[test]
fn canceling_the_sync_token_interrupts_a_fetch() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let git_started = temp_dir.path().join("git-started");
    let fake_git = temp_dir.path().join("git");
    fs::write(
        &fake_git,
        format!("#!/bin/sh\ntouch '{}'\nsleep 30\n", git_started.display()),
    )
    .expect("write fake git");
    fs::set_permissions(&fake_git, fs::Permissions::from_mode(0o755)).expect("chmod fake git");
    run_jj_in(
        &repo_path,
        &[
            "git",
            "remote",
            "add",
            "origin",
            "https://example.invalid/repo.git",
        ],
    );
    run_jj_in(
        &repo_path,
        &[
            "config",
            "set",
            "--repo",
            "git.executable-path",
            fake_git.to_str().expect("utf-8 path"),
        ],
    );
    let repo = Repo::open(&repo_path).expect("open repo");
    let sync = repo.sync_token();

    let outcome = thread::scope(|scope| {
        let fetch = scope.spawn(|| repo.git_fetch(&sync));
        let deadline = Instant::now() + Duration::from_secs(10);
        while !git_started.exists() {
            assert!(Instant::now() < deadline, "fetch never reached git");
            thread::sleep(Duration::from_millis(20));
        }
        sync.cancel();
        fetch.join().expect("fetch thread")
    });

    let error = outcome.expect_err("fetch should be canceled");
    assert!(matches!(error, JayError::Canceled), "{error}");
}

#[test]
fn pulling_a_bookmark_tracks_it_on_the_remote_it_came_from() {
    let (dir, repo_path, bare) = repo_with_remote("upstream");
    let main = git_stdout(&bare, &["rev-parse", "refs/heads/main"]);
    run_git(&bare, &["update-ref", "refs/heads/feature", main.trim()]);
    let repo = Repo::open(&repo_path).expect("open repo");
    repo.git_pull_bookmark("feature", &repo.sync_token())
        .expect("pull feature");
    assert_eq!(
        find_bookmark(&repo, "feature").tracked_remotes,
        ["upstream"]
    );

    let fork = dir.path().join("fork.git");
    run_git(dir.path(), &["init", "--bare", fork.to_str().unwrap()]);
    run_jj_in(
        &repo_path,
        &["git", "remote", "add", "fork", fork.to_str().unwrap()],
    );
    for (key, remote) in [("git.fetch", "upstream"), ("git.push", "fork")] {
        run_jj_in(&repo_path, &["config", "set", "--repo", key, remote]);
    }
    run_git(
        &fork,
        &[
            "fetch",
            "-q",
            bare.to_str().unwrap(),
            "main:refs/heads/other",
        ],
    );
    run_jj_in(&repo_path, &["git", "fetch", "--remote", "fork"]);
    let tree = git_stdout(&bare, &["rev-parse", "main^{tree}"]);
    let upstream_other = git_stdout(
        &bare,
        &[
            "-c",
            "user.name=Upstream",
            "-c",
            "user.email=upstream@example.com",
            "commit-tree",
            tree.trim(),
            "-p",
            main.trim(),
            "-m",
            "upstream other",
        ],
    );
    run_git(
        &bare,
        &["update-ref", "refs/heads/other", upstream_other.trim()],
    );
    let repo = Repo::open(&repo_path).expect("reopen repo");
    repo.git_pull_bookmark("other", &repo.sync_token())
        .expect("pull other");
    let other = find_bookmark(&repo, "other");
    assert_eq!(other.tracked_remotes, ["upstream"]);
    assert_eq!(other.description.trim(), "upstream other");
}
