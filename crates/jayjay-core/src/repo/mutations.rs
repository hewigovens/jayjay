use jj_lib::repo::Repo as _;

use super::Repo;
use super::support::block_on_result;
use crate::types::*;

impl Repo {
    pub fn describe(&self, rev: &str, message: &str) -> CoreResult<()> {
        self.with_resolved_commit_transaction(rev, "describe", true, |_, commit, repo_mut| {
            self.rewrite_commit_description(repo_mut, commit, message, "describe")
        })
    }

    pub fn new_change(&self, parent_rev: &str, message: &str) -> CoreResult<()> {
        let mut args = vec!["new", parent_rev];
        if !message.is_empty() {
            args.extend(["-m", message]);
        }
        self.run_jj_reload(&args)
    }

    pub fn squash(&self, rev: &str, into: Option<&str>) -> CoreResult<()> {
        self.with_resolved_commit_transaction(rev, "squash", true, |repo, commit, repo_mut| {
            let dest = if let Some(into_rev) = into {
                self.resolve_commit(repo, into_rev)?
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

            let parent_tree = self.load_parent_tree(repo, commit, "parent tree")?;
            let source = jj_lib::rewrite::CommitWithSelection {
                selected_tree: commit.tree(),
                parent_tree,
                commit: commit.clone(),
            };

            let result = block_on_result(
                "squash",
                jj_lib::rewrite::squash_commits(repo_mut, &[source], &dest, false),
            )?;

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
                let write = squashed.commit_builder.set_description(combined).write();
                block_on_result("write squashed", write)?;
            }

            Ok(())
        })
    }

    /// Switch the working copy to point at an existing revision (`jj edit`).
    pub fn edit(&self, rev: &str) -> CoreResult<()> {
        self.run_jj_reload(&["edit", rev])
    }

    pub fn abandon(&self, rev: &str) -> CoreResult<()> {
        self.with_resolved_commit_transaction(rev, "abandon", true, |_, commit, repo_mut| {
            repo_mut.record_abandoned_commit(commit);
            Ok(())
        })
    }

    pub fn rebase(&self, rev: &str, dest: &str) -> CoreResult<()> {
        self.with_repo_transaction("rebase", true, |repo, repo_mut| {
            let commit = self.resolve_commit(repo, rev)?;
            let dest_commit = self.resolve_commit(repo, dest)?;
            let rebase =
                jj_lib::rewrite::rebase_commit(repo_mut, commit, vec![dest_commit.id().clone()]);
            block_on_result("rebase", rebase)?;
            Ok(())
        })
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
