use std::path::PathBuf;

use super::Repo;
use crate::types::*;

impl Repo {
    /// `jj commit -m <message>` = describe @ + new empty change on top.
    pub fn jj_commit(&self, message: &str) -> CoreResult<()> {
        let mut cmd = std::process::Command::new("jj");
        cmd.current_dir(&self.path);
        cmd.args(["commit", "-m", message]);
        let output = cmd.output().map_err(|e| CoreError::Internal {
            message: format!("run jj commit: {e}"),
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CoreError::Internal {
                message: format!("jj commit failed: {stderr}"),
            });
        }
        self.reload()
    }

    /// List submodule paths that have uncommitted changes.
    pub fn dirty_submodules(&self) -> CoreResult<Vec<String>> {
        // Parse .gitmodules to find submodule paths
        let gitmodules_path = self.path.join(".gitmodules");
        if !gitmodules_path.exists() {
            return Ok(vec![]);
        }

        let output = std::process::Command::new("git")
            .current_dir(&self.path)
            .args(["submodule", "status"])
            .output()
            .map_err(|e| CoreError::Internal {
                message: format!("git submodule status: {e}"),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut dirty = Vec::new();
        for line in stdout.lines() {
            let line = line.trim();
            // Lines starting with + or - indicate dirty/out-of-date submodules
            if line.starts_with('+') || line.starts_with('-') {
                // Format: "+<hash> <path> (<desc>)" or "-<hash> <path>"
                let parts: Vec<&str> = line[1..].trim().splitn(3, ' ').collect();
                if parts.len() >= 2 {
                    let submodule_path = parts[1];
                    // Check if the submodule actually has uncommitted changes
                    let sub_abs = self.path.join(submodule_path);
                    if has_dirty_workdir(&sub_abs) {
                        dirty.push(submodule_path.to_owned());
                    }
                }
            }
        }
        Ok(dirty)
    }

    /// Commit changes in dirty submodules, then do `jj commit`.
    pub fn commit_with_submodules(&self, message: &str) -> CoreResult<()> {
        // First, commit any dirty submodules
        let dirty = self.dirty_submodules()?;
        for sub_path in &dirty {
            let abs = self.path.join(sub_path);
            let output = std::process::Command::new("git")
                .current_dir(&abs)
                .args(["add", "."])
                .output()
                .map_err(|e| CoreError::Internal {
                    message: format!("git add in {sub_path}: {e}"),
                })?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(CoreError::Internal {
                    message: format!("git add in {sub_path}: {stderr}"),
                });
            }
            let output = std::process::Command::new("git")
                .current_dir(&abs)
                .args(["commit", "-m", message])
                .output()
                .map_err(|e| CoreError::Internal {
                    message: format!("git commit in {sub_path}: {e}"),
                })?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                // "nothing to commit" is ok
                if !stderr.contains("nothing to commit") {
                    return Err(CoreError::Internal {
                        message: format!("git commit in {sub_path}: {stderr}"),
                    });
                }
            }
        }

        // Now jj commit (snapshot will pick up updated submodule pointers)
        self.jj_commit(message)
    }

    /// Get jj configuration as a list of key=value pairs.
    pub fn jj_config(&self) -> CoreResult<String> {
        let output = std::process::Command::new("jj")
            .current_dir(&self.path)
            .args(["config", "list"])
            .output()
            .map_err(|e| CoreError::Internal {
                message: format!("jj config list: {e}"),
            })?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Get jj config file path.
    pub fn jj_config_path(&self) -> CoreResult<String> {
        let output = std::process::Command::new("jj")
            .current_dir(&self.path)
            .args(["config", "path", "--user"])
            .output()
            .map_err(|e| CoreError::Internal {
                message: format!("jj config path: {e}"),
            })?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get a summary of the working copy diff for AI message generation.
    pub fn diff_summary(&self) -> CoreResult<String> {
        let output = std::process::Command::new("jj")
            .current_dir(&self.path)
            .args(["diff", "--stat"])
            .output()
            .map_err(|e| CoreError::Internal {
                message: format!("jj diff --stat: {e}"),
            })?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    /// Returns a message describing what happened (warnings, errors, or success).
    /// For new bookmarks, uses `--named name=rev` which auto-tracks.
    pub fn git_push(&self, bookmark: &str) -> CoreResult<String> {
        let mut cmd = std::process::Command::new("jj");
        cmd.current_dir(&self.path);
        cmd.args(["git", "push"]);
        if !bookmark.is_empty() {
            cmd.args(["--bookmark", bookmark]);
        }
        let output = cmd.output().map_err(|e| CoreError::Internal {
            message: format!("run jj git push: {e}"),
        })?;
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if !output.status.success() {
            // If refused because new remote bookmark, retry with tracking
            if !bookmark.is_empty() && stderr.contains("Refusing to create new remote bookmark") {
                // Track the bookmark first, then retry
                let _ = std::process::Command::new("jj")
                    .current_dir(&self.path)
                    .args(["bookmark", "track", &format!("{bookmark}@origin")])
                    .output();
                let retry = std::process::Command::new("jj")
                    .current_dir(&self.path)
                    .args(["git", "push", "--bookmark", bookmark])
                    .output()
                    .map_err(|e| CoreError::Internal {
                        message: format!("retry push: {e}"),
                    })?;
                let retry_stderr = String::from_utf8_lossy(&retry.stderr).to_string();
                let retry_stdout = String::from_utf8_lossy(&retry.stdout).to_string();
                if !retry.status.success() {
                    return Err(CoreError::Internal {
                        message: format!("git push failed: {retry_stderr}"),
                    });
                }
                self.reload()?;
                return Ok(combine_output(&retry_stdout, &retry_stderr));
            }
            return Err(CoreError::Internal {
                message: format!("git push failed: {stderr}"),
            });
        }
        self.reload()?;
        Ok(combine_output(&stdout, &stderr))
    }

    /// Returns a message describing what happened.
    pub fn git_fetch(&self, remote: &str) -> CoreResult<String> {
        let mut cmd = std::process::Command::new("jj");
        cmd.current_dir(&self.path);
        cmd.args(["git", "fetch"]);
        if !remote.is_empty() {
            cmd.args(["--remote", remote]);
        }
        let output = cmd.output().map_err(|e| CoreError::Internal {
            message: format!("run jj git fetch: {e}"),
        })?;
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if !output.status.success() {
            return Err(CoreError::Internal {
                message: format!("git fetch failed: {stderr}"),
            });
        }
        self.reload()?;
        Ok(combine_output(&stdout, &stderr))
    }
}

fn combine_output(stdout: &str, stderr: &str) -> String {
    let mut parts = Vec::new();
    let s = stdout.trim();
    let e = stderr.trim();
    if !s.is_empty() { parts.push(s); }
    if !e.is_empty() { parts.push(e); }
    if parts.is_empty() { "Done.".to_owned() } else { parts.join("\n") }
}

fn has_dirty_workdir(path: &PathBuf) -> bool {
    std::process::Command::new("git")
        .current_dir(path)
        .args(["status", "--porcelain"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false)
}
