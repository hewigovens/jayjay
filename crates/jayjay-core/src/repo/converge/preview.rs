use std::sync::Arc;

use futures::{StreamExt as _, TryStreamExt as _};
use jj_lib::backend::{ChangeId, Signature};
use jj_lib::commit::Commit;
use jj_lib::converge::{
    ConvergedAttribute, TruncatedEvolutionGraph, converge_change, find_divergent_changes,
};
use jj_lib::merged_tree::MergedTree;
use jj_lib::object_id::{HexPrefix, ObjectId as _};
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::revset::{ResolvedRevsetExpression, RevsetStreamExt as _};

use super::super::Repo;
use super::super::support::{block_on, block_on_result};
use crate::types::*;

pub(super) struct ConvergeAnalysis {
    pub graph: TruncatedEvolutionGraph,
    pub plan: ConvergePlan,
}

impl Repo {
    pub fn converge_preview(&self, rev: &str) -> JayResult<ConvergePlan> {
        self.refresh_working_copy()?;
        let repo = self.get_repo();
        let expression = if !rev.is_empty() && HexPrefix::try_from_reverse_hex(rev).is_some() {
            format!("change_id({rev})")
        } else {
            rev.to_owned()
        };
        let selection = self.evaluate_revset(&repo, &expression)?;
        let commits: Vec<Commit> = block_on_result(
            "resolve convergence change",
            selection.stream().commits(repo.store()).try_collect(),
        )?;
        let first = commits.first().ok_or_else(|| JayError::RevNotFound {
            rev: rev.to_owned(),
        })?;
        if commits
            .iter()
            .any(|commit| commit.change_id() != first.change_id())
        {
            return Err(JayError::internal(format!(
                "converge {rev}: expected one change ID"
            )));
        }
        ConvergeAnalysis::new(self, &repo, first.change_id())?
            .map(|analysis| analysis.plan)
            .ok_or_else(|| JayError::ConvergeNotDivergent {
                change_id: first.change_id().reverse_hex(),
            })
    }
}

impl ConvergeAnalysis {
    // Orchestration follows jj 0.47.1 cli/src/commands/converge.rs.
    pub fn new(
        handle: &Repo,
        repo: &Arc<ReadonlyRepo>,
        change_id: &ChangeId,
    ) -> JayResult<Option<Self>> {
        let change = change_id.reverse_hex();
        let selection = handle.evaluate_revset(repo, &format!("change_id({change})"))?;
        let ids = block_on_result(
            "find convergence versions",
            selection.stream().try_collect(),
        )?;
        let mut changes = block_on_result(
            "find divergent change",
            find_divergent_changes(repo, ResolvedRevsetExpression::commits(ids)),
        )?;
        let Some(commits) = changes.remove(change_id) else {
            return Ok(None);
        };
        let graph = block_on_result(
            "read convergence evolution",
            TruncatedEvolutionGraph::new(repo.clone(), commits),
        )?;
        let proposal = block_on_result(
            "preview convergence",
            converge_change(&graph, None, None, None, None),
        )?;
        let contents_source = match (&proposal.parents, &proposal.tree) {
            (ConvergedAttribute::Solved(parents), Some(tree)) => graph
                .divergent_commits()
                .iter()
                .find(|commit| {
                    commit.parent_ids() == parents.as_slice() && *commit.tree_ids() == tree.tree_ids
                })
                .map(|commit| commit.id().hex()),
            _ => None,
        };
        let conflicted_paths = proposal
            .tree
            .map(|tree| Self::conflicted_paths(&tree.to_merged_tree(repo.store())))
            .transpose()?;
        let author = match proposal.author {
            ConvergedAttribute::Solved(author) => ConvergeAuthorAttribute::Solved {
                value: Self::author(author),
            },
            ConvergedAttribute::Unsolved { .. } => ConvergeAuthorAttribute::Unsolved,
        };
        let description = match proposal.description {
            ConvergedAttribute::Solved(value) => ConvergeDescriptionAttribute::Solved { value },
            ConvergedAttribute::Unsolved { .. } => ConvergeDescriptionAttribute::Unsolved,
        };
        let parents = match proposal.parents {
            ConvergedAttribute::Solved(ids) => ConvergeParentsAttribute::Solved {
                value: ids.iter().map(|id| id.hex()).collect(),
            },
            ConvergedAttribute::Unsolved { .. } => ConvergeParentsAttribute::Unsolved,
        };
        let (versions, effects) = Self::impact(handle, &graph)?;
        let mut basis: Vec<String> = graph
            .divergent_commit_ids()
            .iter()
            .map(|id| id.hex())
            .collect();
        basis.sort();
        Ok(Some(Self {
            graph,
            plan: ConvergePlan {
                change_id: change,
                versions,
                author,
                description,
                parents,
                conflicted_paths,
                contents_source,
                effects,
                basis,
            },
        }))
    }

    fn impact(
        handle: &Repo,
        graph: &TruncatedEvolutionGraph,
    ) -> JayResult<(Vec<ConvergeVersion>, ConvergeEffects)> {
        let repo = graph.repo();
        let change = graph.change_id().reverse_hex();
        let mut effects = ConvergeEffects::default();
        let mut versions = Vec::new();
        for commit in graph.divergent_commits() {
            let is_immutable = handle.is_commit_immutable(repo, commit)?;
            if is_immutable {
                effects.blocked_reason = Some(format!(
                    "change {change}: commit {} is immutable and cannot be rewritten",
                    commit.id().hex()
                ));
            }
            let workspaces = Self::workspaces(repo, commit);
            let bookmarks: Vec<String> = repo
                .view()
                .local_bookmarks()
                .filter(|(_, target)| target.present_adds().any(|id| id == commit.id()))
                .map(|(name, _)| name.as_str().to_owned())
                .collect();
            effects.workspaces.extend(workspaces.iter().cloned());
            effects.bookmarks.extend(bookmarks.iter().cloned());
            versions.push(ConvergeVersion {
                commit_id: commit.id().hex(),
                description: commit.description().to_owned(),
                author: Self::author(commit.author().clone()),
                parents: commit.parent_ids().iter().map(|id| id.hex()).collect(),
                is_immutable,
                workspaces,
                bookmarks,
            });
        }
        let selected = ResolvedRevsetExpression::commits(graph.divergent_commit_ids().clone());
        let descendants = selected
            .descendants()
            .minus(&selected)
            .evaluate(repo.as_ref())
            .map_err(JayError::internal)?;
        let descendants: Vec<Commit> = block_on_result(
            "find convergence descendants",
            descendants.stream().commits(repo.store()).try_collect(),
        )?;
        for commit in descendants {
            if handle.is_commit_immutable(repo, &commit)? {
                effects.blocked_reason = Some(format!(
                    "change {change}: descendant {} is immutable and cannot be rewritten",
                    commit.id().hex()
                ));
            }
            effects.workspaces.extend(Self::workspaces(repo, &commit));
            effects.descendants.push(ConvergeDescendant {
                commit_id: commit.id().hex(),
                description: Self::subject(&commit),
            });
        }
        effects.bookmarks.sort();
        effects.bookmarks.dedup();
        effects.workspaces.sort();
        effects.workspaces.dedup();
        Ok((versions, effects))
    }

    fn subject(commit: &Commit) -> String {
        commit
            .description()
            .lines()
            .next()
            .unwrap_or_default()
            .to_owned()
    }

    fn workspaces(repo: &ReadonlyRepo, commit: &Commit) -> Vec<String> {
        repo.view()
            .wc_commit_ids()
            .iter()
            .filter(|(_, id)| *id == commit.id())
            .map(|(name, _)| name.as_str().to_owned())
            .collect()
    }

    pub(super) fn author(author: Signature) -> ConvergeAuthor {
        ConvergeAuthor {
            name: author.name,
            email: author.email,
            timestamp_millis: author.timestamp.timestamp.0,
            timezone_offset_minutes: author.timestamp.tz_offset,
        }
    }

    pub fn conflicted_paths(tree: &MergedTree) -> JayResult<Vec<String>> {
        block_on(tree.conflicts().collect::<Vec<_>>())
            .into_iter()
            .map(|(path, value)| {
                value.map_err(|error| {
                    JayError::internal(format!(
                        "read convergence conflict {}: {error}",
                        path.as_internal_file_string()
                    ))
                })?;
                Ok(path.as_internal_file_string().to_owned())
            })
            .collect()
    }
}
