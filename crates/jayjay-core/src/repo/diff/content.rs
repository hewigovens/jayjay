use futures::StreamExt as _;
use jj_lib::matchers::{EverythingMatcher, FilesMatcher};
use jj_lib::merged_tree::TreeDiffEntry;
use pollster::FutureExt as _;

use super::entry::{
    diff_hunk_type, first_diff_content, materialize_diff_content, resolve_diff_values,
};
use super::{Repo, TreePair, rename::detect_renames};
use crate::types::*;

impl Repo {
    /// Walk tree diff and return file list WITHOUT content (fast).
    pub(super) fn diff_file_list(&self, trees: &TreePair) -> CoreResult<Vec<DiffHunk>> {
        let path_converter = self.path_converter();
        let mut diff_stream = trees.before.diff_stream(&trees.after, &EverythingMatcher);
        let mut files = Vec::new();

        while let Some(TreeDiffEntry { path, values }) = diff_stream.next().block_on() {
            let values = resolve_diff_values(&path, values)?;
            files.push(DiffHunk {
                path: path_converter.format_file_path(&path),
                old_path: None,
                old_content: None,
                new_content: None,
                hunk_type: diff_hunk_type(&values),
            });
        }
        detect_renames(&mut files);
        Ok(files)
    }

    /// Walk tree diff and return all hunks WITH content.
    pub(super) fn diff_all_files(&self, trees: &TreePair) -> CoreResult<Vec<DiffHunk>> {
        let path_converter = self.path_converter();
        let mut diff_stream = trees.before.diff_stream(&trees.after, &EverythingMatcher);
        let mut diff = Vec::new();

        while let Some(TreeDiffEntry { path, values }) = diff_stream.next().block_on() {
            let values = resolve_diff_values(&path, values)?;
            let content = materialize_diff_content(trees, &path, values)?;
            diff.push(DiffHunk {
                path: path_converter.format_file_path(&path),
                old_path: None,
                old_content: content.old_content,
                new_content: content.new_content,
                hunk_type: content.hunk_type,
            });
        }
        detect_renames(&mut diff);
        Ok(diff)
    }

    /// Materialize a single file between two trees.
    pub(super) fn diff_single_file(&self, trees: &TreePair, path: &str) -> CoreResult<DiffHunk> {
        let path_converter = self.path_converter();
        let repo_path = self.parse_repo_path(path)?;
        let matcher = FilesMatcher::new(std::iter::once(repo_path.as_ref()));
        let Some((entry_path, content)) = first_diff_content(trees, &matcher)? else {
            return Err(CoreError::Internal {
                message: format!("file not found in diff: {path}"),
            });
        };

        Ok(DiffHunk {
            path: path_converter.format_file_path(&entry_path),
            old_path: None,
            old_content: content.old_content,
            new_content: content.new_content,
            hunk_type: content.hunk_type,
        })
    }
}
