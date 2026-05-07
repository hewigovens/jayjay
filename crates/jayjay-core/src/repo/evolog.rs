use futures::StreamExt as _;
use jj_lib::evolution::CommitEvolutionEntry;
use jj_lib::evolution::walk_predecessors;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;
use pollster::FutureExt as _;

use super::Repo;
use crate::types::*;

impl Repo {
    /// Evolution history of a single change. Most recent rewrite first.
    pub fn evolog(&self, rev: &str) -> CoreResult<Vec<EvologEntry>> {
        let repo = self.get_repo();
        let head = self.resolve_commit(&repo, rev)?;
        let mut entries = Vec::new();
        let stream = walk_predecessors(repo.as_ref(), &[head.id().clone()]);
        futures::pin_mut!(stream);
        while let Some(result) = stream.as_mut().next().block_on() {
            let entry = result.map_err(|e| CoreError::Internal {
                message: format!("walk evolog: {e}"),
            })?;
            entries.push(to_dto(&entry));
        }
        Ok(entries)
    }
}

fn to_dto(entry: &CommitEvolutionEntry) -> EvologEntry {
    let commit = &entry.commit;
    let change_id = encode_reverse_hex(commit.change_id().as_bytes());
    let commit_id = commit.id().hex();
    let (timestamp_millis, operation) = match &entry.operation {
        Some(op) => {
            let meta = op.metadata();
            (meta.time.start.timestamp.0, meta.description.clone())
        }
        None => (commit.author().timestamp.timestamp.0, "rewrite".to_owned()),
    };
    let description = commit.description().lines().next().unwrap_or("").to_owned();
    EvologEntry {
        change_id,
        commit_id,
        timestamp_millis,
        operation,
        description,
    }
}
