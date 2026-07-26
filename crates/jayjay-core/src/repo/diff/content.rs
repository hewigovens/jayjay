use futures::StreamExt as _;
use jj_lib::matchers::{EverythingMatcher, FilesMatcher};
use jj_lib::merged_tree::TreeDiffEntry;
use pollster::FutureExt as _;

use super::entry::{
    compute_review_identity, diff_hunk_type, first_diff_content, materialize_diff_content,
    materialize_file_bytes, resolve_diff_values,
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
            let path_str = path.as_internal_file_string();
            let projection = match formats::path_projection(path_str, DiffProjectionMode::Raw) {
                formats::PathProjection::None => None,
                formats::PathProjection::Ready(projection) => Some(projection),
                formats::PathProjection::ContentGated => {
                    let (old, new) = materialize_file_bytes(trees, &path, values.clone())?;
                    formats::projection_for_input(
                        formats::FormatInput {
                            path: path_str,
                            old: old.as_deref(),
                            new: new.as_deref(),
                        },
                        DiffProjectionMode::Raw,
                    )
                }
            };
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

    /// Per-file stats over the displayed card list: the content-free walk supplies the cards and their rename pairing, then each card's sides are materialized in its effective display mode and dropped after counting, so no blob outlives its own card.
    pub(super) fn diff_file_stats_walk(
        &self,
        trees: &TreePair,
        ignore_whitespace: bool,
    ) -> CoreResult<Vec<FileDiffStats>> {
        let files = self.diff_file_list(trees)?;
        let mut stats = Vec::with_capacity(files.len());
        for file in files {
            let mode = crate::projection::request_mode(file.projection.as_ref(), false)
                .unwrap_or(DiffProjectionMode::Raw);
            let mut hunk = self.diff_single_file_with_mode(trees, &file.path, mode)?;
            if let Some(old_path) = file.old_path.as_deref() {
                hunk.old = self.diff_single_file_with_mode(trees, old_path, mode)?.old;
            }
            stats.push(super::hunk_line_stats(&hunk, ignore_whitespace));
        }
        Ok(stats)
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
