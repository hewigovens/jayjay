use std::fmt::{Display, Write};

use futures::StreamExt as _;
use jayjay_primitives::hex_sha256;
use jj_lib::backend::TreeValue;
use jj_lib::conflicts::materialize_tree_value;
use jj_lib::matchers::Matcher;
use jj_lib::merge::{Diff, MergedTreeValue};
use jj_lib::merged_tree::TreeDiffEntry;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::Repo as _;
use jj_lib::repo_path::{RepoPath, RepoPathBuf};
use pollster::FutureExt as _;

use super::{
    TreePair,
    materialize::{
        ImagePreviewResult, extract_image_preview, git_lfs_object_placeholder,
        git_lfs_pointer_placeholder, is_image_path, materialized_to_string,
        parse_binary_placeholder_size, parse_git_lfs_pointer, preview_placeholder,
    },
};
use crate::repo::support::block_on_result;
use crate::types::*;

pub(super) struct DiffContent {
    pub(super) old_content: Option<String>,
    pub(super) new_content: Option<String>,
    pub(super) old_preview: Option<DiffPreview>,
    pub(super) new_preview: Option<DiffPreview>,
    pub(super) hunk_type: HunkType,
}

pub(super) fn diff_hunk_type(values: &Diff<MergedTreeValue>) -> HunkType {
    match (values.before.is_absent(), values.after.is_absent()) {
        (true, false) => HunkType::Added,
        (false, true) => HunkType::Removed,
        _ => HunkType::Modified,
    }
}

/// Stable content identity from the merge's blob IDs (rebase-invariant).
pub(super) fn compute_review_identity(values: &Diff<MergedTreeValue>) -> String {
    let mut buf = String::new();
    let _ = write!(
        &mut buf,
        "before:{}|after:{}",
        side_repr(&values.before),
        side_repr(&values.after)
    );
    hex_sha256(buf.as_bytes())
}

fn side_repr(merge: &MergedTreeValue) -> String {
    let mut parts = Vec::new();
    for value in merge.iter() {
        parts.push(match value {
            Some(TreeValue::File { id, executable, .. }) => {
                format!("f:{}:{}", id.hex(), if *executable { "x" } else { "-" })
            }
            Some(TreeValue::Symlink(target)) => format!("s:{target}"),
            Some(TreeValue::Tree(id)) => format!("t:{}", id.hex()),
            Some(TreeValue::GitSubmodule(id)) => format!("m:{}", id.hex()),
            None => "absent".to_string(),
        });
    }
    parts.join(",")
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

    if is_image_path(path.as_internal_file_string()) {
        let old_result = extract_image_preview(path, old_value)?;
        let new_result = extract_image_preview(path, new_value)?;
        let old_content = image_side_content(&old_result, hunk_type, Side::Old);
        let new_content = image_side_content(&new_result, hunk_type, Side::New);
        let old_preview = image_side_preview(old_result);
        let new_preview = image_side_preview(new_result);
        return Ok(DiffContent {
            old_content,
            new_content,
            old_preview,
            new_preview,
            hunk_type,
        });
    }

    let (old_content, new_content) = normalize_git_lfs_content(
        materialized_to_string(path, old_value)?,
        materialized_to_string(path, new_value)?,
    );

    Ok(DiffContent {
        old_content,
        new_content,
        old_preview: None,
        new_preview: None,
        hunk_type,
    })
}

#[derive(Clone, Copy)]
enum Side {
    Old,
    New,
}

fn image_side_content(
    result: &ImagePreviewResult,
    hunk_type: HunkType,
    side: Side,
) -> Option<String> {
    match result {
        ImagePreviewResult::Image(preview) => Some(preview_placeholder(preview)),
        ImagePreviewResult::GitLfsPointer(pointer) => Some(git_lfs_pointer_placeholder(pointer)),
        ImagePreviewResult::None => match (side, hunk_type) {
            (Side::Old, HunkType::Added) => None,
            (Side::New, HunkType::Removed) => None,
            _ => Some("<binary file>".to_owned()),
        },
    }
}

fn image_side_preview(result: ImagePreviewResult) -> Option<DiffPreview> {
    match result {
        ImagePreviewResult::Image(preview) => Some(preview),
        ImagePreviewResult::GitLfsPointer(_) | ImagePreviewResult::None => None,
    }
}

pub(super) fn first_diff_content(
    trees: &TreePair,
    matcher: &dyn Matcher,
) -> CoreResult<Option<(RepoPathBuf, DiffContent, String)>> {
    let mut diff_stream = trees.before.diff_stream(&trees.after, matcher);
    let Some(TreeDiffEntry { path, values }) = diff_stream.next().block_on() else {
        return Ok(None);
    };
    let values = resolve_diff_values(&path, values)?;
    let identity = compute_review_identity(&values);
    let content = materialize_diff_content(trees, &path, values)?;
    Ok(Some((path, content, identity)))
}

fn normalize_git_lfs_content(
    mut old_content: Option<String>,
    mut new_content: Option<String>,
) -> (Option<String>, Option<String>) {
    let old_pointer = old_content.as_deref().and_then(parse_git_lfs_pointer);
    let new_pointer = new_content.as_deref().and_then(parse_git_lfs_pointer);

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
