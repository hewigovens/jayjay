// Shared jj fixture builders for integration tests across the workspace.
// Used by jayjay-gpui's component tests and (eventually) jayjay-core's
// real_jj_repo.rs.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

/// Run `jj` rooted at `repo` and panic on non-zero exit.
pub fn run_jj_in(repo: &Path, args: &[&str]) -> Output {
    let mut cmd = Command::new("jj");
    cmd.arg("-R").arg(repo).args(args);
    finish(cmd, args)
}

fn finish(mut cmd: Command, args: &[&str]) -> Output {
    let output = cmd
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

/// Linear history: 3 describing changes ending in a working copy with two new
/// files. Mirrors the same-shaped fixture in `shell/justfile:ui-test-setup`
/// so behavior parity with the SwiftUI UI tests is intentional.
pub struct LinearFixture {
    _tmp: TempDir,
    pub path: PathBuf,
}

impl LinearFixture {
    pub fn build() -> Self {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let path = tmp.path().join("repo");

        let mut init = Command::new("jj");
        init.arg("git").arg("init").arg("--colocate").arg(&path);
        finish(init, &["git", "init", "--colocate", "<repo>"]);
        configure_user(&path);

        write(&path, "README.md", "# Sample project\n");
        run_jj_in(&path, &["describe", "-m", "initial"]);

        run_jj_in(&path, &["new", "-m", "add hello"]);
        write(&path, "hello.txt", "hello\n");

        run_jj_in(&path, &["new", "-m", "add feature"]);
        write(&path, "feature.txt", "feature\n");
        run_jj_in(&path, &["bookmark", "create", "main", "-r", "@"]);

        run_jj_in(&path, &["new"]);
        write(&path, "wip1.txt", "wip 1\n");
        write(&path, "wip2.txt", "wip 2\n");

        Self { _tmp: tmp, path }
    }
}

fn configure_user(repo: &Path) {
    run_jj_in(
        repo,
        &["config", "set", "--repo", "user.name", "Test User"],
    );
    run_jj_in(
        repo,
        &["config", "set", "--repo", "user.email", "test@example.com"],
    );
}

fn write(repo: &Path, rel: &str, contents: &str) {
    fs::write(repo.join(rel), contents).unwrap_or_else(|err| panic!("write {rel}: {err}"));
}
