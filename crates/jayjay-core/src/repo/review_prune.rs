use std::collections::HashSet;
use std::path::Path;

use jayjay_primitives::ReviewStoreSummary;
use jayjay_review::ReviewStore;

use crate::types::*;

/// The changes whose review data is still useful: those mutable in one of the given repositories.
pub struct ReviewPrune {
    keep: HashSet<String>,
}

impl ReviewPrune {
    /// Any repository that fails to open, local or SSH, aborts: a missing path may be an unmounted volume, so its review data must stay.
    pub fn for_repositories(paths: &[impl AsRef<Path>]) -> JayResult<Self> {
        let mut keep = HashSet::new();
        for path in paths.iter().map(AsRef::as_ref) {
            let changes = crate::Repo::open(path)
                .and_then(|repo| repo.log("mutable()"))
                .map_err(|error| {
                    JayError::review(format!("Couldn't open {}: {error}", path.display()))
                })?;
            keep.extend(changes.into_iter().map(|change| change.change_id.id));
        }
        Ok(Self { keep })
    }

    pub fn preview(&self, store: &ReviewStore) -> ReviewStoreSummary {
        store.prunable(|change_id| self.keep.contains(change_id))
    }

    pub fn apply(&self, store: &mut ReviewStore) -> JayResult<ReviewStoreSummary> {
        store
            .prune(|change_id| self.keep.contains(change_id))
            .map_err(|error| JayError::review(format!("Couldn't prune review data: {error}")))
    }
}
