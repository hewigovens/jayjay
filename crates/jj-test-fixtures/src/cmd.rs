use std::path::Path;
use std::process::{Command, Output};

/// Run `jj` rooted at `repo` and panic on non-zero exit.
pub fn run_jj_in(repo: &Path, args: &[&str]) -> Output {
    let mut cmd = Command::new("jj");
    cmd.arg("-R").arg(repo).args(args);
    finish(cmd, args)
}

/// Build a fresh colocated jj repo at `path` (must not exist yet).
pub fn init_colocated(path: &Path) {
    let mut init = Command::new("jj");
    init.arg("git").arg("init").arg("--colocate").arg(path);
    finish(init, &["git", "init", "--colocate", "<repo>"]);
}

/// Set a deterministic test identity so commit hashes are reproducible.
pub fn configure_test_user(repo: &Path) {
    run_jj_in(repo, &["config", "set", "--repo", "user.name", "Test User"]);
    run_jj_in(
        repo,
        &["config", "set", "--repo", "user.email", "test@example.com"],
    );
}

pub(crate) fn finish(mut cmd: Command, args: &[&str]) -> Output {
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
