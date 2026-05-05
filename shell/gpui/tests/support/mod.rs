//! Test fixture helpers for gpui-shell integration tests.
//!
//! Mirrors the SwiftUI shell's `ui-test-setup` justfile target: builds a
//! deterministic jj repo on disk so `RepoViewModel` and friends have something
//! real to load. Each fixture lives in a `tempfile::TempDir` so tests are
//! hermetic and parallel-safe.
//!
//! Tests skip gracefully when `jj` and `git` aren't on PATH — keeps `cargo test`
//! green on machines without a jj install.
//!
//! Allow `dead_code` because individual integration test files only consume a
//! subset of these helpers; Rust would otherwise warn per-test-binary.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use tempfile::TempDir;

/// Whether the host has both `jj` and `git` available. Tests that need a
/// fixture should call this and `return` early when false so the suite stays
/// green on bare runners (e.g. before CI installs jj).
pub fn jj_available() -> bool {
    cmd_runs("jj") && cmd_runs("git")
}

fn cmd_runs(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn run_jj(args: &[&str]) -> Output {
    let output = Command::new("jj")
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("failed to spawn jj {args:?}: {err}"));
    if !output.status.success() {
        panic!(
            "jj {args:?} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
    output
}

/// A `simple` jj repo: three describing changes ending in a working copy with
/// two new files. Mirrors the `simple` fixture in `shell/justfile:ui-test-setup`
/// so behavior parity with the SwiftUI UI tests is intentional.
pub struct SimpleFixture {
    _tmp: TempDir,
    pub path: PathBuf,
}

impl SimpleFixture {
    pub fn build() -> Self {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let path = tmp.path().join("simple");
        let repo = path.to_str().expect("repo path utf-8");

        run_jj(&["git", "init", "--colocate", repo]);
        configure_user(&path);

        write(&path, "README.md", "# Sample project\n");
        run_jj(&["-R", repo, "describe", "-m", "initial"]);

        run_jj(&["-R", repo, "new", "-m", "add hello"]);
        write(&path, "hello.txt", "hello\n");

        run_jj(&["-R", repo, "new", "-m", "add feature"]);
        write(&path, "feature.txt", "feature\n");
        run_jj(&["-R", repo, "bookmark", "create", "main", "-r", "@"]);

        run_jj(&["-R", repo, "new"]);
        write(&path, "wip1.txt", "wip 1\n");
        write(&path, "wip2.txt", "wip 2\n");

        Self { _tmp: tmp, path }
    }
}

fn configure_user(repo: &Path) {
    let repo_str = repo.to_str().expect("repo path utf-8");
    run_jj(&[
        "-R", repo_str, "config", "set", "--repo", "user.name", "Test User",
    ]);
    run_jj(&[
        "-R",
        repo_str,
        "config",
        "set",
        "--repo",
        "user.email",
        "test@example.com",
    ]);
}

fn write(repo: &Path, rel: &str, contents: &str) {
    fs::write(repo.join(rel), contents).unwrap_or_else(|err| panic!("write {rel}: {err}"));
}
