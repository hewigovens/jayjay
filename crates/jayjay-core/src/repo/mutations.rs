use jj_lib::repo::Repo as _;
use pollster::FutureExt as _;

use super::Repo;
use crate::types::*;

impl Repo {
    pub fn describe(&self, rev: &str, message: &str) -> CoreResult<()> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;

        let mut tx = repo.start_transaction();
        tx.repo_mut()
            .rewrite_commit(&commit)
            .set_description(message)
            .write()
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("describe: {e}"),
            })?;
        tx.repo_mut()
            .rebase_descendants()
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("rebase descendants: {e}"),
            })?;
        let new_repo = tx
            .commit("describe")
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("commit tx: {e}"),
            })?;
        self.set_repo(new_repo);
        Ok(())
    }

    pub fn new_change(&self, parent_rev: &str, message: &str) -> CoreResult<()> {
        let repo = self.get_repo();
        let parent = self.resolve_commit(&repo, parent_rev)?;

        let mut tx = repo.start_transaction();
        let tree = parent.tree();
        let new_commit = tx
            .repo_mut()
            .new_commit(vec![parent.id().clone()], tree)
            .set_description(message)
            .write()
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("new change: {e}"),
            })?;
        let wc_name = self.workspace_name.clone();
        tx.repo_mut()
            .edit(wc_name, &new_commit)
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("edit working copy: {e}"),
            })?;
        let new_repo = tx
            .commit("new change")
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("commit tx: {e}"),
            })?;
        self.set_repo(new_repo);
        Ok(())
    }

    pub fn squash(&self, rev: &str, into: Option<&str>) -> CoreResult<()> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;

        let dest = if let Some(into_rev) = into {
            self.resolve_commit(&repo, into_rev)?
        } else {
            let parent_ids = commit.parent_ids();
            if parent_ids.is_empty() {
                return Err(CoreError::Internal {
                    message: "cannot squash root commit".to_owned(),
                });
            }
            repo.store()
                .get_commit(&parent_ids[0])
                .map_err(|e| CoreError::Internal {
                    message: format!("get parent: {e}"),
                })?
        };

        let parent_tree = commit
            .parent_tree(repo.as_ref())
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("parent tree: {e}"),
            })?;

        let source = jj_lib::rewrite::CommitWithSelection {
            selected_tree: commit.tree(),
            parent_tree,
            commit: commit.clone(),
        };

        let mut tx = repo.start_transaction();
        let result = jj_lib::rewrite::squash_commits(tx.repo_mut(), &[source], &dest, false)
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("squash: {e}"),
            })?;

        if let Some(squashed) = result {
            let source_desc = commit.description().trim();
            let dest_desc = dest.description().trim();
            let combined = if source_desc.is_empty() {
                dest_desc.to_owned()
            } else if dest_desc.is_empty() {
                source_desc.to_owned()
            } else {
                format!("{dest_desc}\n{source_desc}")
            };
            squashed
                .commit_builder
                .set_description(combined)
                .write()
                .block_on()
                .map_err(|e| CoreError::Internal {
                    message: format!("write squashed: {e}"),
                })?;
        }

        tx.repo_mut()
            .rebase_descendants()
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("rebase descendants: {e}"),
            })?;
        let new_repo = tx
            .commit("squash")
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("commit tx: {e}"),
            })?;
        self.set_repo(new_repo);
        Ok(())
    }

    pub fn abandon(&self, rev: &str) -> CoreResult<()> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;

        let mut tx = repo.start_transaction();
        tx.repo_mut().record_abandoned_commit(&commit);
        tx.repo_mut()
            .rebase_descendants()
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("rebase descendants: {e}"),
            })?;
        let new_repo = tx
            .commit("abandon")
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("commit tx: {e}"),
            })?;
        self.set_repo(new_repo);
        Ok(())
    }

    pub fn rebase(&self, rev: &str, dest: &str) -> CoreResult<()> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        let dest_commit = self.resolve_commit(&repo, dest)?;

        let mut tx = repo.start_transaction();
        jj_lib::rewrite::rebase_commit(tx.repo_mut(), commit, vec![dest_commit.id().clone()])
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("rebase: {e}"),
            })?;
        tx.repo_mut()
            .rebase_descendants()
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("rebase descendants: {e}"),
            })?;
        let new_repo = tx
            .commit("rebase")
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("commit tx: {e}"),
            })?;
        self.set_repo(new_repo);
        Ok(())
    }

    pub fn restore_files(&self, rev: &str, paths: &[String]) -> CoreResult<()> {
        self.refresh_working_copy()?;

        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        let is_wc = repo
            .view()
            .get_wc_commit_id(self.workspace_name.as_ref())
            .is_some_and(|id| id == commit.id());

        if is_wc {
            for p in paths {
                let abs_path = self.path.join(p);
                if abs_path.exists() {
                    std::fs::remove_file(&abs_path)
                        .or_else(|_| std::fs::remove_dir_all(&abs_path))
                        .map_err(|e| CoreError::Internal {
                            message: format!("delete {p}: {e}"),
                        })?;
                }
            }
            self.refresh_working_copy()?;
        } else {
            let old_tree = commit.tree();
            let parent_tree = commit
                .parent_tree(repo.as_ref())
                .block_on()
                .map_err(|e| CoreError::Internal {
                    message: format!("load parent tree: {e}"),
                })?;

            let repo_paths: Vec<jj_lib::repo_path::RepoPathBuf> = paths
                .iter()
                .map(|p| {
                    jj_lib::repo_path::RepoPathBuf::parse_fs_path(&self.path, &self.path, p)
                        .map_err(|e| CoreError::Internal {
                            message: format!("invalid path {p}: {e}"),
                        })
                })
                .collect::<CoreResult<Vec<_>>>()?;

            let matcher =
                jj_lib::matchers::FilesMatcher::new(repo_paths.iter().map(|p| p.as_ref()));
            let new_tree = jj_lib::rewrite::restore_tree(
                &parent_tree,
                &old_tree,
                "parent".to_owned(),
                "current".to_owned(),
                &matcher,
            )
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("restore tree: {e}"),
            })?;

            let mut tx = repo.start_transaction();
            tx.repo_mut()
                .rewrite_commit(&commit)
                .set_tree(new_tree)
                .write()
                .block_on()
                .map_err(|e| CoreError::Internal {
                    message: format!("rewrite commit: {e}"),
                })?;
            tx.repo_mut()
                .rebase_descendants()
                .block_on()
                .map_err(|e| CoreError::Internal {
                    message: format!("rebase descendants: {e}"),
                })?;
            let new_repo = tx
                .commit("restore files")
                .block_on()
                .map_err(|e| CoreError::Internal {
                    message: format!("commit tx: {e}"),
                })?;
            self.set_repo(new_repo);
        }
        Ok(())
    }

    /// Add paths to .gitignore and untrack them via `jj file untrack`.
    pub fn ignore_and_untrack(&self, paths: &[String]) -> CoreResult<()> {
        // Append to .gitignore
        let gitignore_path = self.path.join(".gitignore");
        let existing = std::fs::read_to_string(&gitignore_path).unwrap_or_default();
        let mut lines_to_add = Vec::new();
        for p in paths {
            // Use the path as-is for the gitignore pattern
            if !existing.lines().any(|line| line.trim() == p.as_str()) {
                lines_to_add.push(p.as_str());
            }
        }
        if !lines_to_add.is_empty() {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&gitignore_path)
                .map_err(|e| CoreError::Internal {
                    message: format!("open .gitignore: {e}"),
                })?;
            // Ensure we start on a new line
            if !existing.is_empty() && !existing.ends_with('\n') {
                writeln!(file).ok();
            }
            for line in &lines_to_add {
                writeln!(file, "{line}").map_err(|e| CoreError::Internal {
                    message: format!("write .gitignore: {e}"),
                })?;
            }
        }

        // Untrack via jj CLI
        let mut cmd = std::process::Command::new("jj");
        cmd.current_dir(&self.path);
        cmd.args(["file", "untrack"]);
        for p in paths {
            cmd.arg(p);
        }
        let output = cmd.output().map_err(|e| CoreError::Internal {
            message: format!("run jj file untrack: {e}"),
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CoreError::Internal {
                message: format!("untrack failed: {stderr}"),
            });
        }
        self.reload()
    }

    /// Split selected files out of a change into a new sibling change.
    /// The selected files stay in the original change; the rest moves to a new
    /// change inserted before it. This matches `jj split --paths`.
    pub fn split(&self, rev: &str, paths: &[String]) -> CoreResult<()> {
        let mut cmd = std::process::Command::new("jj");
        cmd.current_dir(&self.path);
        cmd.args(["split", "--revision", rev]);
        for p in paths {
            cmd.arg(p);
        }
        let output = cmd.output().map_err(|e| CoreError::Internal {
            message: format!("run jj split: {e}"),
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CoreError::Internal {
                message: format!("split failed: {stderr}"),
            });
        }
        self.reload()
    }
}
