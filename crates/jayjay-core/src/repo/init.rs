use std::path::Path;

use super::environment;
use crate::types::*;

pub fn init_jj_git_repo(path: &Path) -> JayResult<()> {
    let output = environment::command(environment::require_jj()?)
        .current_dir(path)
        .args(["git", "init"])
        .output()
        .map_err(|e| JayError::Internal {
            message: format!("jj git init: {e}"),
        })?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(JayError::Internal {
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

    #[test]
    fn initializes_jj_git_repo() {
        let tmp = tempfile::tempdir().expect("tempdir");
        init_jj_git_repo(tmp.path()).expect("init repo");

        assert!(tmp.path().join(".jj").exists());
    }

    #[test]
    fn preserves_existing_repository_init_error() {
        let tmp = tempfile::tempdir().expect("tempdir");
        init_jj_git_repo(tmp.path()).expect("init repo");

        let err = init_jj_git_repo(tmp.path()).expect_err("reinitialization should fail");
        let JayError::Internal { message } = err else {
            panic!("unexpected error kind");
        };
        assert!(message.starts_with("jj git init: "));
        assert!(message.contains("The target repo already exists"));
    }
}
