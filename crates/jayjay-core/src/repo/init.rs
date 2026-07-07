use std::path::Path;

use super::environment;
use crate::types::*;

pub fn init_jj_git_repo(path: &Path) -> CoreResult<()> {
    let status = environment::check_jj_environment();
    if !status.is_installed || status.path.is_empty() {
        return Err(CoreError::Internal {
            message: "jj is not installed. Install Jujutsu and try again.".to_owned(),
        });
    }

    let output = environment::command(&status.path)
        .current_dir(path)
        .args(["git", "init"])
        .output()
        .map_err(|e| CoreError::Internal {
            message: format!("jj git init: {e}"),
        })?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(CoreError::Internal {
        message: if stderr.is_empty() {
            "jj git init failed".to_owned()
        } else {
            format!("jj git init: {stderr}")
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    #[test]
    fn initializes_jj_git_repo() {
        let tmp = tempfile::tempdir().expect("tempdir");
        init_jj_git_repo(tmp.path()).expect("init repo");

        assert!(tmp.path().join(".jj").exists());
    }

    #[test]
    fn preserves_git_worktree_init_error() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let main = tmp.path().join("main");
        let worktree = tmp.path().join("worktree");
        fs::create_dir(&main).expect("create main repo");
        run_git(&main, &["init"]);
        fs::write(main.join("README.md"), "hello\n").expect("write readme");
        run_git(&main, &["add", "."]);
        run_git(
            &main,
            &[
                "-c",
                "user.name=Test User",
                "-c",
                "user.email=test@example.com",
                "commit",
                "-m",
                "init",
            ],
        );
        run_git(
            &main,
            &["worktree", "add", worktree.to_str().expect("worktree path")],
        );

        let err = init_jj_git_repo(&worktree).expect_err("init worktree should fail");
        let CoreError::Internal { message } = err else {
            panic!("unexpected error kind");
        };
        assert!(message.contains("Cannot create a colocated jj repo inside a Git worktree"));
        assert!(message.contains("Run `jj git init` in the main Git repository"));
    }

    fn run_git(repo: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .unwrap_or_else(|err| panic!("run git {args:?}: {err}"));
        assert!(
            output.status.success(),
            "git {args:?} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
