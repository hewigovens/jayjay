use std::fmt::Display;

use futures::StreamExt as _;
use jj_lib::conflicts::materialize_tree_value;
use jj_lib::matchers::Matcher;
use jj_lib::merge::{Diff, MergedTreeValue};
use jj_lib::merged_tree::TreeDiffEntry;
use jj_lib::repo::Repo as _;
use jj_lib::repo_path::{RepoPath, RepoPathBuf};
use pollster::FutureExt as _;

use super::{TreePair, materialize::materialized_to_string};
use crate::repo::support::block_on_result;
use crate::types::*;

pub(super) struct DiffContent {
    pub(super) old_content: Option<String>,
    pub(super) new_content: Option<String>,
    pub(super) hunk_type: HunkType,
}

pub(super) fn diff_hunk_type(values: &Diff<MergedTreeValue>) -> HunkType {
    match (values.before.is_absent(), values.after.is_absent()) {
        (true, false) => HunkType::Added,
        (false, true) => HunkType::Removed,
        _ => HunkType::Modified,
    }
}

pub(super) fn resolve_diff_values<E>(
    path: &RepoPath,
    values: Result<Diff<MergedTreeValue>, E>,
) -> CoreResult<Diff<MergedTreeValue>>
where
    E: Display,
{
    values.map_err(|e| CoreError::Internal {
        message: format!("tree diff {}: {e}", path.as_internal_file_string()),
    })
}

pub(super) fn materialize_diff_content(
    trees: &TreePair,
    path: &RepoPath,
    values: Diff<MergedTreeValue>,
) -> CoreResult<DiffContent> {
    let hunk_type = diff_hunk_type(&values);
    let old_value = materialize_tree_value(
        trees.repo.store(),
        path,
        values.before,
        trees.before.labels(),
    );
    let old_value = block_on_result(
        &format!("materialize old {}", path.as_internal_file_string()),
        old_value,
    )?;
    let new_value =
        materialize_tree_value(trees.repo.store(), path, values.after, trees.after.labels());
    let new_value = block_on_result(
        &format!("materialize new {}", path.as_internal_file_string()),
        new_value,
    )?;

    Ok(DiffContent {
        old_content: materialized_to_string(path, old_value)?,
        new_content: materialized_to_string(path, new_value)?,
        hunk_type,
    })
}

pub(super) fn first_diff_content(
    trees: &TreePair,
    matcher: &dyn Matcher,
) -> CoreResult<Option<(RepoPathBuf, DiffContent)>> {
    let mut diff_stream = trees.before.diff_stream(&trees.after, matcher);
    let Some(TreeDiffEntry { path, values }) = diff_stream.next().block_on() else {
        return Ok(None);
    };
    let values = resolve_diff_values(&path, values)?;
    let content = materialize_diff_content(trees, &path, values)?;
    Ok(Some((path, content)))
}
