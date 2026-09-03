use std::sync::Arc;

use futures::StreamExt as _;
use jj_lib::backend::CommitId;
use jj_lib::commit::Commit;
use jj_lib::op_walk;
use jj_lib::repo::{ReadonlyRepo, Repo as JjRepo};
use jj_lib::revset::ResolvedRevsetExpression;

use super::super::Repo;
use super::super::support::{block_on, on_worker_stack};
use crate::types::*;

const MAX_REWRITE_HOPS: usize = 100;
const MAX_OP_SCAN: usize = 1000;

impl Repo {
    // A mutation must not target a hidden commit: a snapshot may have just rewritten the commit the shell selected, so follow the operation log's predecessor records to its visible successor.
    pub(crate) fn follow_rewrites(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: Commit,
        rev: &str,
    ) -> CoreResult<Commit> {
        let revset = ResolvedRevsetExpression::all()
            .evaluate(repo.as_ref())
            .map_err(|e| CoreError::Internal {
                message: format!("visibility revset: {e}"),
            })?;
        let contains = revset.containing_fn();
        let is_visible = |id: &CommitId| {
            block_on(contains(id)).map_err(|e| CoreError::Internal {
                message: format!("visibility check: {e}"),
            })
        };
        if is_visible(commit.id())? {
            return Ok(commit);
        }
        let mut current = commit.id().clone();
        for _ in 0..MAX_REWRITE_HOPS {
            let successors = self.direct_successors(repo, &current)?;
            current = match successors.as_slice() {
                [] => break,
                [next] => next.clone(),
                _ => {
                    return Err(CoreError::Internal {
                        message: format!(
                            "{rev} was rewritten into multiple commits; reselect the target"
                        ),
                    });
                }
            };
            if is_visible(&current)? {
                return repo
                    .store()
                    .get_commit(&current)
                    .map_err(|e| CoreError::Internal {
                        message: format!("get successor: {e}"),
                    });
            }
        }
        Err(CoreError::Internal {
            message: format!("{rev} is hidden and has no visible successor; reselect the target"),
        })
    }

    fn direct_successors(
        &self,
        repo: &Arc<ReadonlyRepo>,
        id: &CommitId,
    ) -> CoreResult<Vec<CommitId>> {
        // Merged operation heads can each record a rewrite of the same commit, so scan the whole capped ancestry and let the fork surface instead of taking the first branch's answer.
        let ops = op_walk::walk_ancestors(std::slice::from_ref(repo.operation())).take(MAX_OP_SCAN);
        on_worker_stack(|| {
            futures::pin_mut!(ops);
            let mut successors: Vec<CommitId> = Vec::new();
            while let Some(op) = block_on(ops.next()) {
                let op = op.map_err(|e| CoreError::Internal {
                    message: format!("walk operations: {e}"),
                })?;
                let Some(map) = &op.store_operation().commit_predecessors else {
                    continue;
                };
                for (successor, predecessors) in map {
                    if predecessors.contains(id) && !successors.contains(successor) {
                        successors.push(successor.clone());
                    }
                }
            }
            Ok(successors)
        })
    }
}
