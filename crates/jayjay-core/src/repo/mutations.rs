use std::collections::HashMap;
use std::sync::Arc;

use futures::TryStreamExt as _;
use jj_lib::absorb::{AbsorbSource, absorb_hunks, split_hunks_to_trees};
use jj_lib::backend::CommitId;
use jj_lib::commit::{Commit, conflict_label_for_commits};
use jj_lib::config::ConfigGetError;
use jj_lib::matchers::{EverythingMatcher, FilesMatcher};
use jj_lib::merge::Merge;
use jj_lib::merged_tree::MergedTree;
use jj_lib::object_id::ObjectId as _;
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::revset::{ResolvedRevsetExpression, RevsetStreamExt as _, UserRevsetExpression};
use jj_lib::rewrite::{
    CommitWithSelection, MoveCommitsLocation, MoveCommitsStats, MoveCommitsTarget, RebaseOptions,
    RebasedCommit, restore_tree, squash_commits,
};
use jj_lib::settings::UserSettings;

use super::Repo;
use super::support::block_on_result;
use crate::types::*;

impl Repo {
    pub fn describe(&self, rev: &str, message: &str) -> CoreResult<()> {
        let _write = self.write_guard()?;
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
        let _write = self.write_guard()?;
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
        let _write = self.write_guard()?;
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
        let revset = self.evaluate_typed_revset(repo.as_ref(), expression)?;
        let ids: Vec<CommitId> = block_on_result("children revset", revset.stream().try_collect())?;
        ids.iter()
            .map(|id| repo.store().get_commit(id))
            .collect::<Result<_, _>>()
            .map_err(|e| CoreError::Internal {
                message: format!("get child: {e}"),
            })
    }

    pub fn squash(&self, rev: &str, into: Option<&str>) -> CoreResult<()> {
        let _write = self.write_guard()?;
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
            let source = CommitWithSelection {
                selected_tree: commit.tree(),
                parent_tree,
                commit: commit.clone(),
            };

            let result =
                block_on_result("squash", squash_commits(repo_mut, &[source], &dest, false))?;

            if let Some(squashed) = result {
                let combined = combined_description(dest.description(), commit.description());
                let write = squashed.commit_builder.set_description(combined).write();
                block_on_result("write squashed", write)?;
            }

            Ok(())
        })
    }

    /// Switch the working copy to point at an existing revision (`jj edit`).
    /// Replicates the full `jj edit` lifecycle: snapshot → edit → rebase → checkout.
    pub fn edit(&self, rev: &str) -> CoreResult<()> {
        let _write = self.write_guard()?;
        self.refresh_working_copy()?;
        self.with_resolved_commit_transaction(rev, "edit", true, |repo, commit, repo_mut| {
            // @ on an immutable commit would let the next snapshot rewrite it.
            self.ensure_commit_mutable(repo, commit, rev)?;
            self.edit_working_copy_commit(repo_mut, commit, "edit")
        })
    }

    pub fn abandon(&self, rev: &str) -> CoreResult<()> {
        let _write = self.write_guard()?;
        self.refresh_working_copy()?;
        self.with_resolved_commit_transaction(rev, "abandon", true, |repo, commit, repo_mut| {
            self.ensure_commit_mutable(repo, commit, rev)?;
            repo_mut.record_abandoned_commit(commit);
            Ok(())
        })
    }

    pub fn abandon_many(&self, revs: &[String]) -> CoreResult<()> {
        let _write = self.write_guard()?;
        require_multiple_revisions(revs, "Abandon selected")?;
        let (repo, commits) = self.snapshot_and_follow_commits(revs)?;
        for (commit, rev) in commits.iter().zip(revs) {
            self.ensure_commit_mutable(&repo, commit, rev)?;
        }
        let mut tx = repo.start_transaction();
        for commit in &commits {
            tx.repo_mut().record_abandoned_commit(commit);
        }
        self.commit_transaction_rebase(tx, "abandon")
    }

    /// Returns the commit id of `rev` after the rebase.
    pub fn rebase(&self, rev: &str, dest: &str, mode: RebaseMode) -> CoreResult<String> {
        let _write = self.write_guard()?;
        self.refresh_working_copy()?;
        let repo = self.get_repo();
        let commit = self.follow_rewrites(&repo, self.resolve_commit(&repo, rev)?, rev)?;
        let dest_commit = self.follow_rewrites(&repo, self.resolve_commit(&repo, dest)?, dest)?;
        let roots = match mode {
            RebaseMode::Source => vec![commit.clone()],
            RebaseMode::Branch => self.branch_roots(&repo, &commit, &dest_commit)?,
        };
        // jj-lib rewrites even an already-in-place commit, which would only record an operation and stale any other checkout of the change.
        if roots
            .iter()
            .all(|root| root.parent_ids() == std::slice::from_ref(dest_commit.id()))
        {
            return Ok(commit.id().hex());
        }
        // Only the moved commits are rewritten; the destination just gains a child and may be immutable.
        for root in &roots {
            let label = if root.id() == commit.id() {
                rev.to_owned()
            } else {
                root.change_id().reverse_hex()
            };
            self.ensure_commit_mutable(&repo, root, &label)?;
        }
        // Descendants follow the rebased commit, so a destination below it forms a cycle that jj-lib panics on; branch roots are never ancestors of the destination.
        if mode == RebaseMode::Source
            && block_on_result(
                "rebase",
                repo.index().is_ancestor(commit.id(), dest_commit.id()),
            )?
        {
            return Err(CoreError::Internal {
                message: format!(
                    "Cannot rebase {rev} onto {dest}: it is the same change or one of its descendants"
                ),
            });
        }
        let root_ids = roots.iter().map(|root| root.id().clone()).collect();
        let stats = self.move_onto(&repo, MoveCommitsTarget::Roots(root_ids), &dest_commit)?;
        Ok(match stats.rebased_commits.get(commit.id()) {
            Some(RebasedCommit::Rewritten(new_commit)) => new_commit.id().hex(),
            _ => commit.id().hex(),
        })
    }

    /// Runs on `repo`, the snapshot the operands were resolved against, so a concurrent operation is merged rather than handed stale ids.
    fn move_onto(
        &self,
        repo: &Arc<ReadonlyRepo>,
        target: MoveCommitsTarget,
        dest: &Commit,
    ) -> CoreResult<MoveCommitsStats> {
        let location = MoveCommitsLocation {
            new_parent_ids: vec![dest.id().clone()],
            new_child_ids: Vec::new(),
            target,
        };
        let mut tx = repo.start_transaction();
        let options = RebaseOptions::default();
        let stats = block_on_result(
            "rebase",
            jj_lib::rewrite::move_commits(tx.repo_mut(), &location, &options),
        )?;
        if tx.repo().has_changes() {
            self.commit_transaction_rebase(tx, "rebase")?;
        }
        Ok(stats)
    }

    /// `roots(dest..rev)`: the bottom of every line of work leading to `rev` that `dest` does not already contain.
    fn branch_roots(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &Commit,
        dest: &Commit,
    ) -> CoreResult<Vec<Commit>> {
        let roots = ResolvedRevsetExpression::commits(vec![dest.id().clone()])
            .range(&ResolvedRevsetExpression::commits(vec![
                commit.id().clone(),
            ]))
            .roots()
            .evaluate(repo.as_ref())
            .map_err(|e| CoreError::internal(format!("branch roots: {e}")))?;
        block_on_result(
            "branch roots",
            roots.stream().commits(repo.store()).try_collect(),
        )
    }

    pub fn rebase_many(&self, revs: &[String], dest: &str) -> CoreResult<()> {
        let _write = self.write_guard()?;
        require_multiple_revisions(revs, "Rebase selected")?;
        let mut targets = revs.to_vec();
        targets.push(dest.to_owned());
        let (repo, mut commits) = self.snapshot_and_follow_commits(&targets)?;
        let dest_commit = commits.pop().expect("destination pushed above");
        // A destination inside or below the selection forms a cycle that jj-lib panics on.
        for commit in &commits {
            if block_on_result(
                "rebase",
                repo.index().is_ancestor(commit.id(), dest_commit.id()),
            )? {
                return Err(CoreError::internal(
                    "Cannot rebase the selection onto one of its own changes or their descendants",
                ));
            }
        }
        for (commit, rev) in commits.iter().zip(revs) {
            self.ensure_commit_mutable(&repo, commit, rev)?;
        }
        // move_commits expects children before parents, which is the order a revset yields.
        let ordered = ResolvedRevsetExpression::commits(
            commits.iter().map(|commit| commit.id().clone()).collect(),
        )
        .evaluate(repo.as_ref())
        .map_err(|e| CoreError::internal(format!("order selection: {e}")))?;
        let ids = block_on_result("order selection", ordered.stream().try_collect())?;
        self.move_onto(&repo, MoveCommitsTarget::Commits(ids), &dest_commit)
            .map(drop)
    }

    /// Squash a newest-first, consecutive linear selection into its oldest change.
    /// Returns the destination's commit id after the squash.
    pub fn squash_many(&self, revs: &[String]) -> CoreResult<String> {
        let _write = self.write_guard()?;
        require_multiple_revisions(revs, "Squash selected")?;
        let (repo, commits) = self.snapshot_and_follow_commits(revs)?;
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
        let (destination, sources) = commits.split_last().expect("validated non-empty");
        for commit in &commits {
            self.ensure_commit_mutable(&repo, commit, &commit.id().hex())?;
        }
        let sources = sources
            .iter()
            .map(|commit| {
                Ok(CommitWithSelection {
                    selected_tree: commit.tree(),
                    parent_tree: self.load_parent_tree(&repo, commit, "parent tree")?,
                    commit: commit.clone(),
                })
            })
            .collect::<CoreResult<Vec<_>>>()?;

        let mut tx = repo.start_transaction();
        let squashed = block_on_result(
            "squash",
            squash_commits(tx.repo_mut(), &sources, destination, false),
        )?;
        if let Some(squashed) = squashed {
            let write = squashed.commit_builder.set_description(message).write();
            block_on_result("write squashed", write)?;
        }
        self.commit_transaction_rebase(tx, "squash")?;

        let repo = self.get_repo();
        let destination =
            self.follow_rewrites(&repo, destination.clone(), &destination.id().hex())?;
        Ok(destination.id().hex())
    }

    /// Create a merge commit with multiple parents (`jj new A B`).
    pub fn merge(&self, parent_revs: &[String]) -> CoreResult<()> {
        let _write = self.write_guard()?;
        require_multiple_revisions(parent_revs, "Merge")?;
        let (repo, parents) = self.snapshot_and_follow_commits(parent_revs)?;
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

        let mut tx = repo.start_transaction();
        let repo_mut = tx.repo_mut();
        let tree = block_on_result(
            "merge",
            jj_lib::rewrite::merge_commit_trees(repo_mut, &parents),
        )?;
        let parent_ids = parents.iter().map(|parent| parent.id().clone()).collect();
        let merge = block_on_result("merge", repo_mut.new_commit(parent_ids, tree).write())?;
        self.edit_working_copy_commit(repo_mut, &merge, "edit working copy")?;
        self.commit_transaction_rebase(tx, "merge")
    }

    /// Duplicate a revision (`jj duplicate`).
    pub fn duplicate(&self, rev: &str) -> CoreResult<()> {
        let _write = self.write_guard()?;
        let (repo, commits) = self.snapshot_and_follow_commits(&[rev.to_owned()])?;
        if commits[0].parent_ids().is_empty() {
            return Err(CoreError::internal("The root change cannot be duplicated"));
        }
        let mut tx = repo.start_transaction();
        let targets = [commits[0].id().clone()];
        let descriptions = HashMap::new();
        let duplicated =
            jj_lib::rewrite::duplicate_commits_onto_parents(tx.repo_mut(), &targets, &descriptions);
        block_on_result("duplicate", duplicated)?;
        self.commit_transaction(tx, "duplicate")
    }

    /// `jj absorb --from rev`: each hunk moves into the mutable ancestor that last touched those lines; hunks without an unambiguous home stay in the source.
    pub fn absorb(&self, rev: &str) -> CoreResult<MutationEffect> {
        let _write = self.write_guard()?;
        let (repo, commits) = self.snapshot_and_follow_commits(&[rev.to_owned()])?;
        // Destinations come from mutable(), so the source is the only rewritten commit left to check.
        self.ensure_commit_mutable(&repo, &commits[0], rev)?;
        let source = block_on_result(
            "absorb",
            AbsorbSource::from_commit(repo.as_ref(), commits[0].clone()),
        )?;
        let destinations = self.resolve_revset(&repo, "mutable()")?;
        let selected = block_on_result(
            "absorb",
            split_hunks_to_trees(repo.as_ref(), &source, &destinations, &EverythingMatcher),
        )?;
        if selected.target_commits.is_empty() {
            return Ok(MutationEffect::Unchanged);
        }
        let mut tx = repo.start_transaction();
        let stats = block_on_result(
            "absorb",
            absorb_hunks(tx.repo_mut(), &source, selected.target_commits),
        )?;
        self.commit_transaction_rebase(
            tx,
            &format!(
                "absorb changes into {} commits",
                stats.rewritten_destinations.len()
            ),
        )?;
        Ok(MutationEffect::Changed)
    }

    /// `jj revert -r rev --onto @`: a new child of `@` whose tree takes back `rev`'s changes.
    pub fn revert_change(&self, rev: &str) -> CoreResult<()> {
        let _write = self.write_guard()?;
        let (repo, commits) = self.snapshot_and_follow_commits(&[rev.to_owned()])?;
        let target = &commits[0];
        let onto = self.working_copy_commit(&repo)?;
        let old_parents = block_on_result("load parents", target.parents())?;
        let old_base_tree = self.load_parent_tree(&repo, target, "parent tree")?;
        let reverted_tree = block_on_result(
            "revert",
            MergedTree::merge(Merge::from_vec(vec![
                (
                    onto.tree(),
                    format!("{} (revert destination)", onto.conflict_label()),
                ),
                (
                    target.tree(),
                    format!("{} (reverted revision)", target.conflict_label()),
                ),
                (
                    old_base_tree,
                    format!(
                        "{} (parents of reverted revision)",
                        conflict_label_for_commits(&old_parents)
                    ),
                ),
            ])),
        )?;
        let description = format!(
            "Revert \"{}\"\n\nThis reverts commit {}.\n",
            target.description().lines().next().unwrap_or(""),
            target.id().hex()
        );
        let mut tx = repo.start_transaction();
        let write = tx
            .repo_mut()
            .new_commit(vec![onto.id().clone()], reverted_tree)
            .set_description(description)
            .write();
        block_on_result("revert", write)?;
        self.commit_transaction_rebase(tx, &format!("revert commit {}", target.id().hex()))
    }

    /// `jj split -r rev -m message -- paths`: the named files become a commit described by `message` that keeps the change id; the rest becomes a new change on top, or a sibling when `parallel`. Workspaces on `rev` move to the remainder.
    pub fn split(
        &self,
        rev: &str,
        paths: &[String],
        message: &str,
        parallel: bool,
    ) -> CoreResult<()> {
        let _write = self.write_guard()?;
        let (repo, commits) = self.snapshot_and_follow_commits(&[rev.to_owned()])?;
        let target = &commits[0];
        self.ensure_commit_mutable(&repo, target, rev)?;
        let repo_paths = self.parse_repo_paths(paths)?;
        let matcher = FilesMatcher::new(repo_paths.iter().map(|path| path.as_ref()));
        let parent_tree = self.load_parent_tree(&repo, target, "parent tree")?;
        let target_tree = target.tree();
        let selected_tree = block_on_result(
            "select split files",
            restore_tree(
                &target_tree,
                &parent_tree,
                "split revision".to_owned(),
                "parents of split revision".to_owned(),
                &matcher,
            ),
        )?;
        if selected_tree.tree_ids() == parent_tree.tree_ids() {
            return Err(CoreError::internal(
                "none of the selected files differ from the parent",
            ));
        }
        let remainder_tree = if parallel {
            block_on_result(
                "remove split files",
                restore_tree(
                    &parent_tree,
                    &target_tree,
                    "parents of split revision".to_owned(),
                    "split revision".to_owned(),
                    &matcher,
                ),
            )?
        } else {
            target_tree
        };
        let message = if message.is_empty() { "split" } else { message };
        let legacy_bookmark_behavior = legacy_split_bookmark_behavior(repo.settings())?;
        let mut tx = repo.start_transaction();
        let first = {
            let mut builder = tx.repo_mut().rewrite_commit(target).detach();
            builder.set_tree(selected_tree).set_description(message);
            block_on_result("write split commit", builder.write(tx.repo_mut()))?
        };
        let second = {
            let parents = if parallel {
                target.parent_ids().to_vec()
            } else {
                vec![first.id().clone()]
            };
            let mut builder = tx.repo_mut().rewrite_commit(target).detach();
            builder.set_parents(parents).set_tree(remainder_tree);
            builder.clear_rewrite_source();
            builder.generate_new_change_id();
            block_on_result("write remainder commit", builder.write(tx.repo_mut()))?
        };
        // With the legacy behavior bookmarks on the split change follow the remainder; otherwise they stay on the part that kept the change id.
        if legacy_bookmark_behavior {
            tx.repo_mut()
                .set_rewritten_commit(target.id().clone(), second.id().clone());
        }
        block_on_result(
            "rebase descendants",
            tx.repo_mut()
                .transform_descendants(vec![target.id().clone()], async |mut rewriter| {
                    match (parallel, legacy_bookmark_behavior) {
                        (true, true) => {
                            rewriter.replace_parent(second.id(), [first.id(), second.id()]);
                        }
                        (true, false) => {
                            rewriter.replace_parent(first.id(), [first.id(), second.id()]);
                        }
                        (false, _) => rewriter.replace_parent(first.id(), [second.id()]),
                    }
                    rewriter.rebase().await?.write().await?;
                    Ok(())
                }),
        )?;
        for (name, wc_commit_id) in repo.view().wc_commit_ids() {
            if wc_commit_id == target.id() {
                block_on_result(
                    "edit working copy",
                    tx.repo_mut().edit(name.clone(), &second),
                )?;
            }
        }
        self.commit_transaction_rebase(tx, &format!("split commit {}", target.id().hex()))
    }

    // Snapshot first, then retarget each selected commit id at its visible successor, as concrete ids and never `@`: a snapshot may have just rewritten the selection, and a late-bound operand would follow a concurrent working-copy move.
    fn snapshot_and_follow_commits(
        &self,
        revs: &[String],
    ) -> CoreResult<(Arc<ReadonlyRepo>, Vec<Commit>)> {
        self.refresh_working_copy()?;
        let repo = self.get_repo();
        let commits = revs
            .iter()
            .map(|rev| self.follow_rewrites(&repo, self.resolve_commit(&repo, rev)?, rev))
            .collect::<CoreResult<_>>()?;
        Ok((repo, commits))
    }

    pub(crate) fn snapshot_and_follow(&self, revs: &[String]) -> CoreResult<Vec<String>> {
        Ok(self
            .snapshot_and_follow_commits(revs)?
            .1
            .iter()
            .map(|commit| commit.id().hex())
            .collect())
    }

    pub(crate) fn snapshot_and_follow_one(&self, rev: &str) -> CoreResult<String> {
        Ok(self.snapshot_and_follow(&[rev.to_owned()])?.remove(0))
    }
}

/// `split.legacy-bookmark-behavior` is defined by the CLI's config, not jj-lib's, so an unset key means the CLI default.
fn legacy_split_bookmark_behavior(settings: &UserSettings) -> CoreResult<bool> {
    match settings.get_bool("split.legacy-bookmark-behavior") {
        Ok(value) => Ok(value),
        Err(ConfigGetError::NotFound { .. }) => Ok(true),
        Err(error) => Err(CoreError::internal(error)),
    }
}

/// The description a squash leaves on its destination: both messages, with an empty side dropped.
pub(super) fn combined_description(destination: &str, source: &str) -> String {
    match (destination.trim(), source.trim()) {
        (destination, "") => destination.to_owned(),
        ("", source) => source.to_owned(),
        (destination, source) => format!("{destination}\n{source}"),
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
