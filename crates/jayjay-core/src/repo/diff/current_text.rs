use std::sync::Arc;

use jj_lib::conflicts::materialize_tree_value;
use jj_lib::merged_tree::MergedTree;
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::repo_path::RepoPath;

use super::{Repo, materialize::materialized_to_content};
use crate::repo::support::block_on_result;
use crate::types::*;

impl Repo {
    /// Uses the same materialization pipeline as diff rendering, so callers like the diff-edit staleness guard compare like-for-like against previously rendered content.
    pub(crate) fn materialize_path_text(
        &self,
        repo: &Arc<ReadonlyRepo>,
        tree: &MergedTree,
        path: &RepoPath,
    ) -> CoreResult<Option<String>> {
        let value = block_on_result(
            &format!("read {}", path.as_internal_file_string()),
            tree.path_value(path),
        )?;
        let materialized = block_on_result(
            &format!("materialize {}", path.as_internal_file_string()),
            materialize_tree_value(repo.store(), path, value, tree.labels()),
        )?;
        Ok(materialized_to_content(path, materialized)?.raw_string())
    }
}
