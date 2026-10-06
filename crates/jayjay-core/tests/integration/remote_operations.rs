#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::thread;
use std::time::{Duration, Instant};

use jayjay_core::{CoreError, Repo};
use jj_test::{LinearFixture, git_stdout, init_jj_repo, run_jj_in};

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
        let fetch = scope.spawn(|| repo.git_fetch("origin", &sync));
        let deadline = Instant::now() + Duration::from_secs(10);
        while !git_started.exists() {
            assert!(Instant::now() < deadline, "fetch never reached git");
            thread::sleep(Duration::from_millis(20));
        }
        sync.cancel();
        fetch.join().expect("fetch thread")
    });

    let error = outcome.expect_err("fetch should be canceled");
    assert!(matches!(error, CoreError::Canceled), "{error}");
}

#[test]
fn canceling_tag_deletion_push_restores_its_local_target() {
    let fixture = LinearFixture::build();
    let remote = fixture.add_bare_origin();
    let repo = Repo::open(&fixture.path).unwrap();
    repo.create_tag("release", "@-").unwrap();
    repo.git_push_tag("release", &repo.sync_token()).unwrap();
    let target = git_stdout(&fixture.path, &["rev-parse", "refs/tags/release"]);
    let git_started = remote.path().join("git-started");
    let fake_git = remote.path().join("fake-git");
    fs::write(
        &fake_git,
        format!("#!/bin/sh\ntouch '{}'\nsleep 30\n", git_started.display()),
    )
    .unwrap();
    fs::set_permissions(&fake_git, fs::Permissions::from_mode(0o755)).unwrap();
    run_jj_in(
        &fixture.path,
        &[
            "config",
            "set",
            "--repo",
            "git.executable-path",
            fake_git.to_str().unwrap(),
        ],
    );
    let sync = repo.sync_token();
    let outcome = thread::scope(|scope| {
        let push = scope.spawn(|| repo.delete_tag_and_push("release", &sync));
        let deadline = Instant::now() + Duration::from_secs(10);
        while !git_started.exists() {
            assert!(Instant::now() < deadline, "push never reached git");
            thread::sleep(Duration::from_millis(20));
        }
        sync.cancel();
        push.join().unwrap()
    });
    assert!(matches!(outcome, Err(CoreError::Canceled)), "{outcome:?}");
    assert_eq!(
        git_stdout(&fixture.path, &["rev-parse", "refs/tags/release"]),
        target
    );
}
