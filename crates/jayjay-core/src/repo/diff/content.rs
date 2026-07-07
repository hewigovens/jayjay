use futures::StreamExt as _;
use jj_lib::matchers::{EverythingMatcher, FilesMatcher};
use jj_lib::merged_tree::TreeDiffEntry;
use pollster::FutureExt as _;

use super::entry::{
    compute_review_identity, diff_hunk_type, first_diff_content, materialize_diff_content,
    resolve_diff_values,
};
use super::{Repo, TreePair, formats, rename::detect_renames};
use crate::types::*;

impl Repo {
    /// Walk tree diff and return file list WITHOUT content (fast).
    pub(super) fn diff_file_list(&self, trees: &TreePair) -> CoreResult<Vec<DiffHunk>> {
        let mut diff_stream = trees.before.diff_stream(&trees.after, &EverythingMatcher);
        let mut files = Vec::new();

        while let Some(TreeDiffEntry { path, values }) = diff_stream.next().block_on() {
            let values = resolve_diff_values(&path, values)?;
            let projection = formats::projection_for_path(
                path.as_internal_file_string(),
                DiffProjectionMode::Raw,
            );
            let review_identity = compute_review_identity(&values, projection.as_ref());
            files.push(DiffHunk {
                path: path.as_internal_file_string().to_owned(),
                old_path: None,
                old: DiffContent::default(),
                new: DiffContent::default(),
                hunk_type: diff_hunk_type(&values),
                review_identity,
                projection,
            });
        }
        detect_renames(&mut files);
        Ok(files)
    }

    /// Walk tree diff and return all hunks WITH content.
    pub(super) fn diff_all_files(&self, trees: &TreePair) -> CoreResult<Vec<DiffHunk>> {
        let mut diff_stream = trees.before.diff_stream(&trees.after, &EverythingMatcher);
        let mut diff = Vec::new();

        while let Some(TreeDiffEntry { path, values }) = diff_stream.next().block_on() {
            let values = resolve_diff_values(&path, values)?;
            let content = materialize_diff_content(
                trees,
                &path,
                values.clone(),
                DiffProjectionMode::Processed,
            )?;
            let review_identity = compute_review_identity(&values, content.projection.as_ref());
            diff.push(DiffHunk {
                path: path.as_internal_file_string().to_owned(),
                old_path: None,
                old: content.old,
                new: content.new,
                hunk_type: content.hunk_type,
                review_identity,
                projection: content.projection,
            });
        }
        detect_renames(&mut diff);
        Ok(diff)
    }

    /// Materialize a single file between two trees.
    pub(super) fn diff_single_file(&self, trees: &TreePair, path: &str) -> CoreResult<DiffHunk> {
        self.diff_single_file_with_mode(trees, path, DiffProjectionMode::Processed)
    }

    pub(super) fn diff_single_file_with_mode(
        &self,
        trees: &TreePair,
        path: &str,
        projection_mode: DiffProjectionMode,
    ) -> CoreResult<DiffHunk> {
        let repo_path = self.parse_repo_path(path)?;
        let matcher = FilesMatcher::new(std::iter::once(repo_path.as_ref()));
        let Some((entry_path, content, review_identity)) =
            first_diff_content(trees, &matcher, projection_mode)?
        else {
            return Err(CoreError::Internal {
                message: format!("file not found in diff: {path}"),
            });
        };

        Ok(DiffHunk {
            path: entry_path.as_internal_file_string().to_owned(),
            old_path: None,
            old: content.old,
            new: content.new,
            hunk_type: content.hunk_type,
            review_identity,
            projection: content.projection,
        })
    }
}
