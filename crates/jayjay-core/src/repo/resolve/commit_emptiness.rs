use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use jj_lib::backend::CommitId;
use jj_lib::commit::Commit;
use jj_lib::repo::ReadonlyRepo;

use super::super::support::block_on;

const CAPACITY: usize = 100_000;

/// Emptiness is fixed by a commit's own tree and parents, so answers stay valid across operations.
#[derive(Default)]
pub(crate) struct CommitEmptiness(RwLock<HashMap<CommitId, bool>>);

impl CommitEmptiness {
    /// Display-only: a backend error reads as non-empty and is not cached.
    #[cfg_attr(feature = "hotpath", hotpath::measure(impl_type = "CommitEmptiness"))]
    pub(crate) fn is_empty(&self, repo: &Arc<ReadonlyRepo>, commit: &Commit) -> bool {
        if let Some(&empty) = self.0.read().unwrap().get(commit.id()) {
            return empty;
        }
        let Ok(empty) = block_on(commit.is_empty(repo.as_ref())) else {
            return false;
        };
        let mut cache = self.0.write().unwrap();
        if cache.len() >= CAPACITY {
            cache.clear();
        }
        cache.insert(commit.id().clone(), empty);
        empty
    }
}
