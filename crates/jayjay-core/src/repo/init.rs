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

    #[test]
    fn initializes_jj_git_repo() {
        let tmp = tempfile::tempdir().expect("tempdir");
        init_jj_git_repo(tmp.path()).expect("init repo");

        assert!(tmp.path().join(".jj").exists());
    }
}
