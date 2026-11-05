use futures::future::try_join_all;
use jj_lib::backend::{ChangeId, CommitId, MillisSinceEpoch, Signature, Timestamp};
use jj_lib::commit::Commit;
use jj_lib::converge::{TreeIdsAndLabels, apply_solution, converge_change};
use jj_lib::merge::MergeBuilder;
use jj_lib::merged_tree::MergedTree;
use jj_lib::object_id::ObjectId as _;
use jj_lib::repo::Repo as _;
use jj_lib::rewrite::merge_commit_trees_no_resolve;

use super::super::Repo;
use super::super::support::block_on_result;
use super::preview::ConvergeAnalysis;
use crate::types::*;

impl Repo {
    pub fn converge_apply(
        &self,
        plan: &ConvergePlan,
        choices: &ConvergeChoices,
    ) -> JayResult<ConvergeOutcome> {
        let _write = self.write_guard()?;
        self.refresh_working_copy()?;
        let repo = self.get_repo();
        let stale = || JayError::ConvergePlanStale {
            change_id: plan.change_id.clone(),
        };
        let change_id = ChangeId::try_from_reverse_hex(&plan.change_id).ok_or_else(stale)?;
        let analysis = ConvergeAnalysis::new(self, &repo, &change_id)?.ok_or_else(stale)?;
        if let Some(reason) = analysis
            .plan
            .effects
            .blocked_reason
            .as_ref()
            .or(plan.effects.blocked_reason.as_ref())
        {
            return Err(JayError::ConvergeBlocked {
                change_id: plan.change_id.clone(),
                reason: reason.clone(),
            });
        }
        if analysis.plan.basis != plan.basis
            || analysis.plan.author != plan.author
            || analysis.plan.description != plan.description
            || analysis.plan.parents != plan.parents
            || analysis.plan.effects.exceeds(&plan.effects)
        {
            return Err(stale());
        }
        let author = analysis.choose(
            "author",
            analysis.plan.author.solved(),
            choices.author.as_deref(),
            |commit| ConvergeAnalysis::author(commit.author().clone()),
        )?;
        let parents = analysis.choose(
            "parents",
            analysis.plan.parents.solved(),
            choices.parents.as_deref(),
            |commit| commit.parent_ids().iter().map(|id| id.hex()).collect(),
        )?;
        let version = match &choices.description {
            Some(DescriptionChoice::FromVersion(id)) => Some(id.as_str()),
            _ => None,
        };
        let description = match &choices.description {
            Some(DescriptionChoice::Custom(text)) => text.clone(),
            _ => analysis.choose(
                "description",
                analysis.plan.description.solved(),
                version,
                |commit| commit.description().to_owned(),
            )?,
        };
        let author = Signature {
            name: author.name,
            email: author.email,
            timestamp: Timestamp {
                timestamp: MillisSinceEpoch(author.timestamp_millis),
                tz_offset: author.timezone_offset_minutes,
            },
        };
        let parents: Vec<CommitId> = parents
            .iter()
            .map(|id| {
                CommitId::try_from_hex(id).ok_or_else(|| {
                    JayError::internal(format!("converge {}: invalid parent {id}", plan.change_id))
                })
            })
            .collect::<JayResult<_>>()?;
        let tree = choices
            .contents
            .as_deref()
            .map(|id| analysis.rebased_tree(id, &parents))
            .transpose()?;
        let proposal = block_on_result(
            "converge chosen parents",
            converge_change(
                &analysis.graph,
                Some(author.clone()),
                Some(description.clone()),
                Some(parents.clone()),
                tree,
            ),
        )?;
        let tree = proposal.tree.ok_or_else(|| {
            JayError::internal(format!(
                "converge {}: failed to converge tree",
                plan.change_id
            ))
        })?;
        let conflicted_paths =
            ConvergeAnalysis::conflicted_paths(&tree.to_merged_tree(repo.store()))?;
        let mut tx = repo.start_transaction();
        let (successor, _) = block_on_result(
            "apply convergence",
            apply_solution(
                author,
                description,
                parents,
                tree,
                change_id,
                analysis.graph.divergent_commit_ids(),
                tx.repo_mut(),
            ),
        )?;
        self.commit_transaction(
            tx,
            &format!(
                "converge divergent change {}",
                plan.change_id.chars().take(8).collect::<String>()
            ),
        )?;
        Ok(ConvergeOutcome {
            successor_commit_id: successor.id().hex(),
            conflicted_paths,
        })
    }
}

impl ConvergeAnalysis {
    fn version(&self, id: &str) -> JayResult<&Commit> {
        self.graph
            .divergent_commits()
            .iter()
            .find(|commit| commit.id().hex() == id)
            .ok_or_else(|| {
                JayError::internal(format!(
                    "converge {}: {id} is not a version",
                    self.plan.change_id
                ))
            })
    }

    fn choose<T: Clone>(
        &self,
        name: &str,
        solved: Option<&T>,
        choice: Option<&str>,
        value: impl Fn(&Commit) -> T,
    ) -> JayResult<T> {
        match (choice, solved) {
            (Some(id), _) => self.version(id).map(value),
            (None, Some(value)) => Ok(value.clone()),
            (None, None) => Err(JayError::internal(format!(
                "converge {}: {name} choice is missing",
                self.plan.change_id
            ))),
        }
    }

    // Mirrors jj-lib's private rebase_tree_onto_solution_parents so a chosen version keeps only its own changes.
    fn rebased_tree(&self, id: &str, parents: &[CommitId]) -> JayResult<TreeIdsAndLabels> {
        let commit = self.version(id)?;
        if commit.parent_ids() == parents {
            return Ok(TreeIdsAndLabels::new(commit.tree()));
        }
        let repo = self.graph.repo();
        block_on_result("rebase chosen contents", async {
            let parent_commits: Vec<Commit> =
                try_join_all(parents.iter().map(|id| repo.store().get_commit_async(id))).await?;
            let terms = [
                (
                    merge_commit_trees_no_resolve(repo.as_ref(), &parent_commits).await?,
                    "converge solution parent(s)".to_owned(),
                ),
                (
                    commit.parent_tree_no_resolve(repo.index()).await?,
                    commit.parents_conflict_label().await?,
                ),
                (commit.tree(), commit.conflict_label()),
            ];
            let tree = MergedTree::merge(MergeBuilder::from_iter(terms).build()).await?;
            Ok::<_, jj_lib::backend::BackendError>(TreeIdsAndLabels::new(tree))
        })
    }
}
