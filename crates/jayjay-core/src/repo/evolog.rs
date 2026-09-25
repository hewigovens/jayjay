use futures::StreamExt as _;
use jj_lib::evolution::CommitEvolutionEntry;
use jj_lib::evolution::walk_predecessors;
use jj_lib::repo::ReadonlyRepo;

use super::Repo;
use super::support::{block_on, on_worker_stack, short_change_id, short_commit_id};
use crate::types::*;

impl Repo {
    /// Evolution history of a single change. Most recent rewrite first.
    pub fn evolog(&self, rev: &str) -> CoreResult<Vec<EvologEntry>> {
        let repo = self.get_repo();
        // A divergent change is shown by commit id, which a restore hides; start from its successor so the reload includes the new version.
        let head = self.follow_rewrites(&repo, self.resolve_commit(&repo, rev)?, rev)?;
        on_worker_stack(|| {
            let mut entries = Vec::new();
            let stream = walk_predecessors(repo.as_ref(), &[head.id().clone()]);
            futures::pin_mut!(stream);
            while let Some(result) = block_on(stream.as_mut().next()) {
                let entry = result.map_err(|e| CoreError::Internal {
                    message: format!("walk evolog: {e}"),
                })?;
                entries.push(to_dto(repo.as_ref(), &entry));
            }
            Ok(entries)
        })
    }
}

fn to_dto(repo: &ReadonlyRepo, entry: &CommitEvolutionEntry) -> EvologEntry {
    let commit = &entry.commit;
    let (timestamp_millis, operation) = match &entry.operation {
        Some(op) => {
            let meta = op.metadata();
            (meta.time.start.timestamp.0, meta.description.clone())
        }
        None => (commit.author().timestamp.timestamp.0, "rewrite".to_owned()),
    };
    let description = commit.description().lines().next().unwrap_or("").to_owned();
    EvologEntry {
        change_id: short_change_id(repo, commit),
        commit_id: short_commit_id(repo, commit),
        timestamp_millis,
        operation,
        description,
    }
}
