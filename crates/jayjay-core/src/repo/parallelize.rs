use std::collections::HashMap;

use futures::TryStreamExt as _;
use jj_lib::backend::CommitId;
use jj_lib::commit::Commit;
use jj_lib::object_id::ObjectId as _;
use jj_lib::repo::Repo as _;
use jj_lib::revset::{ResolvedRevsetExpression, RevsetStreamExt as _};

use super::Repo;
use super::mutations::require_multiple_revisions;
use super::support::block_on_result;
use crate::types::*;

impl Repo {
    /// Replicates jj 0.46.0's `cli/src/commands/parallelize.rs`; jj-lib has no parallelize of its own.
    pub fn parallelize(&self, revs: &[String]) -> JayResult<MutationEffect> {
        let _write = self.write_guard()?;
        require_multiple_revisions(revs, "Parallelize selected")?;
        let (repo, commits) = self.snapshot_and_follow_commits(revs)?;
        let ordered = ResolvedRevsetExpression::commits(
            commits.iter().map(|commit| commit.id().clone()).collect(),
        )
        .evaluate(repo.as_ref())
        .map_err(|e| JayError::internal(format!("order selection: {e}")))?;
        let targets: Vec<Commit> = block_on_result(
            "order selection",
            ordered.stream().commits(repo.store()).try_collect(),
        )?;
        for target in &targets {
            self.ensure_commit_mutable(&repo, target, &target.id().hex())?;
        }

        let mut new_target_parents: HashMap<CommitId, Vec<CommitId>> = HashMap::new();
        let mut new_child_parents: HashMap<CommitId, Vec<CommitId>> = HashMap::new();
        for commit in targets.iter().rev() {
            let target_parents = replaced_parents(commit.parent_ids(), &new_target_parents);
            new_target_parents.insert(commit.id().clone(), target_parents);

            let mut child_parents = Vec::new();
            for old_parent in commit.parent_ids() {
                if let Some(parents) = new_child_parents.get(old_parent) {
                    extend_unique(&mut child_parents, parents);
                }
            }
            child_parents.push(commit.id().clone());
            new_child_parents.insert(commit.id().clone(), child_parents);
        }

        let target_ids: Vec<CommitId> = targets.iter().map(|commit| commit.id().clone()).collect();
        let mut tx = repo.start_transaction();
        block_on_result(
            "parallelize",
            tx.repo_mut()
                .transform_descendants(target_ids, async |mut rewriter| {
                    let old_commit = rewriter.old_commit().clone();
                    if let Some(new_parents) = new_target_parents.get(old_commit.id()) {
                        rewriter.set_new_rewritten_parents(new_parents);
                    } else if old_commit
                        .parent_ids()
                        .iter()
                        .any(|id| new_child_parents.contains_key(id))
                    {
                        let new_parents =
                            replaced_parents(old_commit.parent_ids(), &new_child_parents);
                        rewriter.set_new_rewritten_parents(&new_parents);
                    }
                    if rewriter.parents_changed() {
                        rewriter.rebase().await?.write().await?;
                    }
                    Ok(())
                }),
        )?;
        // `1 | 3` with `2` outside the set rewrites nothing; report it instead of recording an empty operation.
        if !tx.repo().has_changes() {
            return Ok(MutationEffect::Unchanged);
        }
        self.commit_transaction_rebase(tx, &format!("parallelize {} commits", targets.len()))?;
        Ok(MutationEffect::Changed)
    }
}

fn replaced_parents(
    parent_ids: &[CommitId],
    replacements: &HashMap<CommitId, Vec<CommitId>>,
) -> Vec<CommitId> {
    let mut parents = Vec::new();
    for id in parent_ids {
        match replacements.get(id) {
            Some(replacement) => extend_unique(&mut parents, replacement),
            None => extend_unique(&mut parents, std::slice::from_ref(id)),
        }
    }
    parents
}

fn extend_unique(parents: &mut Vec<CommitId>, ids: &[CommitId]) {
    for id in ids {
        if !parents.contains(id) {
            parents.push(id.clone());
        }
    }
}
