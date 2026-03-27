use std::fmt::Display;

use futures::StreamExt as _;
use jj_lib::conflicts::materialize_tree_value;
use jj_lib::matchers::Matcher;
use jj_lib::merge::{Diff, MergedTreeValue};
use jj_lib::merged_tree::TreeDiffEntry;
use jj_lib::repo::Repo as _;
use jj_lib::repo_path::{RepoPath, RepoPathBuf};
use pollster::FutureExt as _;

use super::{
    TreePair,
    materialize::{
        git_lfs_object_placeholder, git_lfs_pointer_placeholder, materialized_to_string,
        parse_binary_placeholder_size, parse_git_lfs_pointer,
    },
};
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
    let (old_content, new_content) = normalize_git_lfs_content(
        materialized_to_string(path, old_value)?,
        materialized_to_string(path, new_value)?,
    );

    Ok(DiffContent {
        old_content,
        new_content,
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

fn normalize_git_lfs_content(
    mut old_content: Option<String>,
    mut new_content: Option<String>,
) -> (Option<String>, Option<String>) {
    let old_pointer = old_content
        .as_deref()
        .and_then(parse_git_lfs_pointer);
    let new_pointer = new_content
        .as_deref()
        .and_then(parse_git_lfs_pointer);

    if let Some(pointer) = old_pointer.as_ref() {
        old_content = Some(git_lfs_pointer_placeholder(pointer));
        if new_content
            .as_deref()
            .and_then(parse_binary_placeholder_size)
            == Some(pointer.size)
        {
            new_content = Some(git_lfs_object_placeholder(pointer));
        }
    }

    if let Some(pointer) = new_pointer.as_ref() {
        new_content = Some(git_lfs_pointer_placeholder(pointer));
        if old_content
            .as_deref()
            .and_then(parse_binary_placeholder_size)
            == Some(pointer.size)
        {
            old_content = Some(git_lfs_object_placeholder(pointer));
        }
    }

    (old_content, new_content)
}

#[cfg(test)]
mod tests {
    use super::normalize_git_lfs_content;

    #[test]
    fn normalizes_pointer_against_binary_placeholder() {
        let (old_content, new_content) = normalize_git_lfs_content(
            Some(
                "version https://git-lfs.github.com/spec/v1\n\
                 oid sha256:496634778d7b9bdbdb4b98b43a08a00ce8d794ed135a0cb1f345bf6febc5b9b4\n\
                 size 742800\n"
                    .to_owned(),
            ),
            Some("<binary file (742800 bytes)>".to_owned()),
        );

        assert_eq!(
            old_content.as_deref(),
            Some("<git lfs pointer sha256:496634778d7b (742800 bytes)>")
        );
        assert_eq!(
            new_content.as_deref(),
            Some("<git lfs object sha256:496634778d7b (742800 bytes)>")
        );
    }
}
