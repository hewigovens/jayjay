use std::sync::Arc;

use futures::TryStreamExt as _;
use jj_lib::backend::CommitId;
use jj_lib::commit::Commit;
use jj_lib::object_id::ObjectId as _;
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::revset::UserRevsetExpression;

use super::Repo;
use super::path_operands::fileset_literal;
use super::support::block_on_result;
use crate::types::*;

impl Repo {
    pub fn describe(&self, rev: &str, message: &str) -> CoreResult<()> {
        // Snapshot disk edits first so rewriting @'s ancestry does not clobber them on checkout.
        self.refresh_working_copy()?;
        self.with_resolved_commit_transaction(rev, "describe", true, |repo, commit, repo_mut| {
            self.ensure_commit_mutable(repo, commit, rev)?;
            self.rewrite_commit_description(repo_mut, commit, message, "describe")
        })
    }

    /// Create a new empty change on top of `parent_rev`.
    /// Replicates the full `jj new` lifecycle:
    /// 1. Snapshot working copy
    /// 2. Create new commit with parent's tree
    /// 3. Edit working copy to point at new commit
    /// 4. Rebase descendants + sync working copy on disk
    pub fn new_change(&self, parent_rev: &str, message: &str) -> CoreResult<()> {
        // Step 1: snapshot working copy (same as jj CLI's workspace_helper)
        self.refresh_working_copy()?;
        // Steps 2-4: create commit, edit @, rebase descendants, checkout
        self.with_resolved_commit_transaction(
            parent_rev,
            "new change",
            true, // always rebase descendants (was false — the bug)
            |_, parent, repo_mut| {
                let tree = parent.tree();
                let new_commit = repo_mut
                    .new_commit(vec![parent.id().clone()], tree)
                    .set_description(message)
                    .write();
                let new_commit = block_on_result("new change", new_commit)?;
                self.edit_working_copy_commit(repo_mut, &new_commit, "edit working copy")
            },
        )
    }

    pub fn new_change_inserted(
        &self,
        rev: &str,
        position: InsertPosition,
        message: &str,
    ) -> CoreResult<()> {
        self.refresh_working_copy()?;
        self.with_resolved_commit_transaction(rev, "new change", true, |repo, target, repo_mut| {
            let (parents, displaced) = match position {
                InsertPosition::Before => {
                    self.ensure_commit_mutable(repo, target, rev)?;
                    let parents = block_on_result("load parents", target.parents())?;
                    (parents, vec![target.clone()])
                }
                InsertPosition::After => {
                    let children = self.children(repo, target)?;
                    for child in &children {
                        self.ensure_commit_mutable(repo, child, &format!("a child of {rev}"))?;
                    }
                    (vec![target.clone()], children)
                }
            };
            let tree = block_on_result(
                "merge parent trees",
                jj_lib::rewrite::merge_commit_trees(repo.as_ref(), &parents),
            )?;
            let parent_ids: Vec<CommitId> =
                parents.iter().map(|parent| parent.id().clone()).collect();
            let new_commit = repo_mut
                .new_commit(parent_ids.clone(), tree)
                .set_description(message)
                .write();
            let new_commit = block_on_result("new change", new_commit)?;
            for commit in displaced {
                let mut new_parents: Vec<CommitId> = Vec::new();
                for parent_id in commit.parent_ids() {
                    let parent_id = if parent_ids.contains(parent_id) {
                        new_commit.id()
                    } else {
                        parent_id
                    };
                    if !new_parents.contains(parent_id) {
                        new_parents.push(parent_id.clone());
                    }
                }
                let rebase = jj_lib::rewrite::rebase_commit(repo_mut, commit, new_parents);
                block_on_result("rebase through new change", rebase)?;
            }
            self.edit_working_copy_commit(repo_mut, &new_commit, "edit working copy")
        })
    }

    fn children(&self, repo: &Arc<ReadonlyRepo>, commit: &Commit) -> CoreResult<Vec<Commit>> {
        let expression = UserRevsetExpression::commit(commit.id().clone()).children();
        let revset = self.evaluate_typed_revset(repo, expression)?;
        let ids: Vec<CommitId> = block_on_result("children revset", revset.stream().try_collect())?;
        ids.iter()
            .map(|id| repo.store().get_commit(id))
            .collect::<Result<_, _>>()
            .map_err(|e| CoreError::Internal {
                message: format!("get child: {e}"),
            })
    }

    pub fn squash(&self, rev: &str, into: Option<&str>) -> CoreResult<()> {
        self.refresh_working_copy()?;
        self.with_resolved_commit_transaction(rev, "squash", true, |repo, commit, repo_mut| {
            self.ensure_commit_mutable(repo, commit, rev)?;
            let dest = if let Some(into_rev) = into {
                self.resolve_commit(repo, into_rev)?
            } else {
                // The mutability gate above already rejected the parentless root commit.
                let first_parent =
                    commit
                        .parent_ids()
                        .first()
                        .ok_or_else(|| CoreError::Internal {
                            message: "cannot squash root commit".to_owned(),
                        })?;
                repo.store()
                    .get_commit(first_parent)
                    .map_err(|e| CoreError::Internal {
                        message: format!("get parent: {e}"),
                    })?
            };
            // Squash rewrites the destination as well as the source.
            self.ensure_commit_mutable(repo, &dest, into.unwrap_or("the parent"))?;

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
    /// Replicates the full `jj edit` lifecycle: snapshot → edit → rebase → checkout.
    pub fn edit(&self, rev: &str) -> CoreResult<()> {
        self.refresh_working_copy()?;
        self.with_resolved_commit_transaction(rev, "edit", true, |repo, commit, repo_mut| {
            // @ on an immutable commit would let the next snapshot rewrite it.
            self.ensure_commit_mutable(repo, commit, rev)?;
            self.edit_working_copy_commit(repo_mut, commit, "edit")
        })
    }

    pub fn abandon(&self, rev: &str) -> CoreResult<()> {
        self.refresh_working_copy()?;
        self.with_resolved_commit_transaction(rev, "abandon", true, |repo, commit, repo_mut| {
            self.ensure_commit_mutable(repo, commit, rev)?;
            repo_mut.record_abandoned_commit(commit);
            Ok(())
        })
    }

    pub fn abandon_many(&self, revs: &[String]) -> CoreResult<()> {
        require_multiple_revisions(revs, "Abandon selected")?;
        let revs = self.snapshot_and_follow(revs)?;
        let mut args = Vec::with_capacity(revs.len() + 2);
        args.extend(["abandon", "--"]);
        args.extend(revs.iter().map(String::as_str));
        self.run_jj_reload(&args)
    }

    /// Returns the commit id of `rev` after the rebase.
    pub fn rebase(&self, rev: &str, dest: &str) -> CoreResult<String> {
        self.refresh_working_copy()?;
        let repo = self.get_repo();
        let commit = self.follow_rewrites(&repo, self.resolve_commit(&repo, rev)?, rev)?;
        let dest_commit = self.follow_rewrites(&repo, self.resolve_commit(&repo, dest)?, dest)?;
        // jj-lib rewrites even an already-in-place commit, which would only record an operation and stale any other checkout of the change.
        if commit.parent_ids() == std::slice::from_ref(dest_commit.id()) {
            return Ok(commit.id().hex());
        }
        // Only the rebased commit is rewritten; the destination just gains a child and may be immutable.
        self.ensure_commit_mutable(&repo, &commit, rev)?;
        // Descendants follow the rebased commit, so a destination below it forms a cycle that jj-lib panics on.
        let dest_is_descendant = block_on_result(
            "rebase",
            repo.index().is_ancestor(commit.id(), dest_commit.id()),
        )?;
        if dest_is_descendant {
            return Err(CoreError::Internal {
                message: format!(
                    "Cannot rebase {rev} onto {dest}: it is the same change or one of its descendants"
                ),
            });
        }
        let mut rebased = None;
        self.with_existing_commit_transaction(
            repo,
            commit,
            "rebase",
            true,
            |_, commit, repo_mut| {
                let rebase = jj_lib::rewrite::rebase_commit(
                    repo_mut,
                    commit.clone(),
                    vec![dest_commit.id().clone()],
                );
                rebased = Some(block_on_result("rebase", rebase)?.id().hex());
                Ok(())
            },
        )?;
        rebased.ok_or_else(|| CoreError::Internal {
            message: "rebase: no rewritten commit".to_owned(),
        })
    }

    pub fn rebase_many(&self, revs: &[String], dest: &str) -> CoreResult<()> {
        require_multiple_revisions(revs, "Rebase selected")?;
        let mut targets = revs.to_vec();
        targets.push(dest.to_owned());
        let followed = self.snapshot_and_follow(&targets)?;
        let (revs, dest) = followed.split_at(revs.len());
        let mut args = Vec::with_capacity(revs.len() * 2 + 3);
        args.push("rebase");
        for rev in revs {
            args.extend(["--revisions", rev]);
        }
        args.extend(["--onto", &dest[0]]);
        self.run_jj_reload(&args)
    }

    /// Squash a newest-first, consecutive linear selection into its oldest change.
    /// Returns the destination's commit id after the squash.
    pub fn squash_many(&self, revs: &[String]) -> CoreResult<String> {
        require_multiple_revisions(revs, "Squash selected")?;
        let commits = self.snapshot_and_follow_commits(revs)?;
        if commits
            .windows(2)
            .any(|pair| pair[0].parent_ids() != std::slice::from_ref(pair[1].id()))
        {
            return Err(CoreError::internal(
                "Squash selected requires a consecutive linear range",
            ));
        }

        let message = commits
            .iter()
            .rev()
            .map(|commit| commit.description().trim())
            .filter(|description| !description.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        let revs: Vec<String> = commits.iter().map(|commit| commit.id().hex()).collect();
        let mut args = Vec::with_capacity(revs.len() * 2 + 4);
        args.push("squash");
        for rev in &revs[..revs.len() - 1] {
            args.extend(["--from", rev]);
        }
        args.extend(["--into", revs.last().expect("validated non-empty")]);
        args.extend(["--message", &message]);
        self.run_jj_reload(&args)?;

        let destination = commits.last().expect("validated non-empty");
        let repo = self.get_repo();
        let destination =
            self.follow_rewrites(&repo, destination.clone(), &destination.id().hex())?;
        Ok(destination.id().hex())
    }

    /// Create a merge commit with multiple parents (`jj new A B`).
    pub fn merge(&self, parent_revs: &[String]) -> CoreResult<()> {
        require_multiple_revisions(parent_revs, "Merge")?;
        let parents = self.snapshot_and_follow_commits(parent_revs)?;
        let repo = self.get_repo();
        for (index, parent) in parents.iter().enumerate() {
            for other in &parents[index + 1..] {
                let related = parent.id() == other.id()
                    || block_on_result("merge", repo.index().is_ancestor(parent.id(), other.id()))?
                    || block_on_result("merge", repo.index().is_ancestor(other.id(), parent.id()))?;
                if related {
                    return Err(CoreError::Internal {
                        message: "Merge requires independent heads; one selected change is an ancestor of another"
                            .to_owned(),
                    });
                }
            }
        }

        let parent_ids: Vec<String> = parents.iter().map(|parent| parent.id().hex()).collect();
        let mut args = vec!["new"];
        args.extend(parent_ids.iter().map(String::as_str));
        self.run_jj_reload(&args)
    }

    /// Duplicate a revision (`jj duplicate`).
    pub fn duplicate(&self, rev: &str) -> CoreResult<()> {
        let rev = self.snapshot_and_follow_one(rev)?;
        self.run_jj_reload(&["duplicate", &rev])
    }

    /// Absorb working-copy hunks into ancestor commits based on blame.
    pub fn absorb(&self, rev: &str) -> CoreResult<()> {
        let rev = self.snapshot_and_follow_one(rev)?;
        self.run_jj_reload(&["absorb", "--from", &rev])
    }

    /// Create a new change that inverts the diff of a prior change on top of `@` (`jj revert`).
    pub fn revert_change(&self, rev: &str) -> CoreResult<()> {
        let rev = self.snapshot_and_follow_one(rev)?;
        let repo = self.get_repo();
        let onto = self.working_copy_commit(&repo)?.id().hex();
        self.run_jj_reload(&["revert", "-r", &rev, "--onto", &onto])
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
        let rev = self.snapshot_and_follow_one(rev)?;
        let mut args = vec!["split", "--revision", rev.as_str()];
        if parallel {
            args.push("--parallel");
        }
        if !message.is_empty() {
            args.extend(["-m", message]);
        } else {
            args.extend(["-m", "split"]);
        }
        // Literal fileset operands after `--`, so an option- or fileset-shaped filename
        // (e.g. `--config=ui.diff-editor=...`) can't become a jj flag or expression.
        let operands: Vec<String> = paths.iter().map(|p| fileset_literal(p)).collect();
        args.push("--");
        args.extend(operands.iter().map(String::as_str));
        self.run_jj_reload(&args)
    }

    // Snapshot first, then retarget each selected commit id at its visible successor, as concrete ids and never `@`: a snapshot may have just rewritten the selection, and a late-bound operand would follow a concurrent working-copy move.
    fn snapshot_and_follow_commits(&self, revs: &[String]) -> CoreResult<Vec<Commit>> {
        self.refresh_working_copy()?;
        let repo = self.get_repo();
        revs.iter()
            .map(|rev| self.follow_rewrites(&repo, self.resolve_commit(&repo, rev)?, rev))
            .collect()
    }

    pub(crate) fn snapshot_and_follow(&self, revs: &[String]) -> CoreResult<Vec<String>> {
        Ok(self
            .snapshot_and_follow_commits(revs)?
            .iter()
            .map(|commit| commit.id().hex())
            .collect())
    }

    pub(crate) fn snapshot_and_follow_one(&self, rev: &str) -> CoreResult<String> {
        Ok(self.snapshot_and_follow(&[rev.to_owned()])?.remove(0))
    }
}

fn require_multiple_revisions(revs: &[String], action: &str) -> CoreResult<()> {
    if revs.len() < 2 {
        return Err(CoreError::internal(format!(
            "{action} requires at least two changes"
        )));
    }
    Ok(())
}
