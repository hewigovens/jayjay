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
        self.commit_transaction_rebase(tx, "describe")
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
        self.commit_transaction(tx, "new change")
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

        let parent_tree =
            commit
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

        self.commit_transaction_rebase(tx, "squash")
    }

    /// Switch the working copy to point at an existing revision (`jj edit`).
    pub fn edit(&self, rev: &str) -> CoreResult<()> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        let mut tx = repo.start_transaction();
        let wc_name = self.workspace_name.clone();
        tx.repo_mut()
            .edit(wc_name, &commit)
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("edit: {e}"),
            })?;
        self.commit_transaction(tx, "edit")
    }

    pub fn abandon(&self, rev: &str) -> CoreResult<()> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;

        let mut tx = repo.start_transaction();
        tx.repo_mut().record_abandoned_commit(&commit);
        self.commit_transaction_rebase(tx, "abandon")
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
        self.commit_transaction_rebase(tx, "rebase")
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
            // Use jj restore which properly reverts files to parent state
            // (modified files get parent content, added files get removed)
            let mut args = vec!["restore", "--from", "@-"];
            args.extend(paths.iter().map(|s| s.as_str()));
            self.run_jj_reload(&args)
        } else {
            let old_tree = commit.tree();
            let parent_tree =
                commit
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
            self.commit_transaction_rebase(tx, "restore files")
        }
    }

    /// Delete files from disk (working copy only). jj will pick up the deletion on next snapshot.
    pub fn delete_files(&self, paths: &[String]) -> CoreResult<()> {
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
        self.refresh_working_copy()
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
        let mut args = vec!["file", "untrack"];
        args.extend(paths.iter().map(|s| s.as_str()));
        self.run_jj_reload(&args)
    }

    /// Move files from a change to working copy using `jj squash --from rev --into @`.
    /// This atomically moves the file changes into @ and removes them from rev.
    pub fn move_to_working_copy(&self, rev: &str, paths: &[String]) -> CoreResult<()> {
        let mut args = vec!["squash", "--from", rev, "--into", "@"];
        args.extend(paths.iter().map(|s| s.as_str()));
        self.run_jj_reload(&args)
    }

    /// Cherry-pick a revision into the current working copy (`jj graft`).
    pub fn graft(&self, rev: &str) -> CoreResult<()> {
        self.run_jj_reload(&["graft", "-r", rev])
    }

    /// Create a merge commit with multiple parents (`jj new A B`).
    pub fn merge(&self, parent_revs: &[String]) -> CoreResult<()> {
        let mut args = vec!["new"];
        args.extend(parent_revs.iter().map(|s| s.as_str()));
        self.run_jj_reload(&args)
    }

    /// Duplicate a revision (`jj duplicate`).
    pub fn duplicate(&self, rev: &str) -> CoreResult<()> {
        self.run_jj_reload(&["duplicate", rev])
    }

    /// Absorb working-copy hunks into ancestor commits based on blame.
    pub fn absorb(&self, rev: &str) -> CoreResult<()> {
        self.run_jj_reload(&["absorb", "--from", rev])
    }

    /// Create a new change that inverts the diff of a prior change (`jj revert`).
    pub fn backout(&self, rev: &str) -> CoreResult<()> {
        self.run_jj_reload(&["revert", "-r", rev, "--insert-after", rev])
    }

    /// Split selected files out of a change into a new change.
    /// When `parallel` is true, creates a sibling (--parallel); otherwise a child.
    pub fn split(
        &self,
        rev: &str,
        paths: &[String],
        message: &str,
        parallel: bool,
    ) -> CoreResult<()> {
        let mut args = vec!["split", "--revision", rev];
        if parallel {
            args.push("--parallel");
        }
        if !message.is_empty() {
            args.extend(["-m", message]);
        } else {
            args.extend(["-m", "split"]);
        }
        args.extend(paths.iter().map(|s| s.as_str()));
        self.run_jj_reload(&args)
    }
}
