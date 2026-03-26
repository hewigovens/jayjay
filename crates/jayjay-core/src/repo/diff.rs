mod content;
mod entry;
mod materialize;
mod rename;

use std::sync::Arc;

use jj_lib::matchers::FilesMatcher;
use jj_lib::merged_tree::MergedTree;
use jj_lib::repo::ReadonlyRepo;

use crate::types::*;

use self::entry::first_diff_content;
use super::Repo;

/// Resolved pair of trees for diffing.
pub(super) struct TreePair {
    pub(super) repo: Arc<ReadonlyRepo>,
    pub(super) before: MergedTree,
    pub(super) after: MergedTree,
}

impl Repo {
    /// Resolve a commit to its parent→commit tree pair.
    fn commit_trees(&self, rev: &str) -> CoreResult<(TreePair, ChangeInfo)> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        let info = self.commit_to_change_info(&repo, &commit);
        let before = self.load_parent_tree(&repo, &commit, "load parent tree")?;
        let after = commit.tree();
        Ok((
            TreePair {
                repo,
                before,
                after,
            },
            info,
        ))
    }

    /// Resolve two revisions to a from→to tree pair.
    fn interdiff_trees(&self, from_rev: &str, to_rev: &str) -> CoreResult<(TreePair, ChangeInfo)> {
        let repo = self.get_repo();
        let from_commit = self.resolve_commit(&repo, from_rev)?;
        let to_commit = self.resolve_commit(&repo, to_rev)?;
        let info = self.commit_to_change_info(&repo, &to_commit);
        let before = from_commit.tree();
        let after = to_commit.tree();
        Ok((
            TreePair {
                repo,
                before,
                after,
            },
            info,
        ))
    }

    fn parse_named_diff_path(
        &self,
        role: &str,
        path: &str,
    ) -> CoreResult<jj_lib::repo_path::RepoPathBuf> {
        self.parse_repo_path(path).map_err(|error| match error {
            CoreError::Internal { message } => CoreError::Internal {
                message: message.replacen("invalid path", &format!("invalid {role} path"), 1),
            },
            other => other,
        })
    }

    /// Fast: returns change info + file list WITHOUT content.
    pub fn show_summary(&self, rev: &str) -> CoreResult<ChangeDetail> {
        let (trees, info) = self.commit_trees(rev)?;
        let diff = self.diff_file_list(&trees)?;
        Ok(ChangeDetail { info, diff })
    }

    /// Full: returns change info + file list WITH content (slow for large changesets).
    pub fn show(&self, rev: &str) -> CoreResult<ChangeDetail> {
        let (trees, info) = self.commit_trees(rev)?;
        let diff = self.diff_all_files(&trees)?;
        Ok(ChangeDetail { info, diff })
    }

    /// Show a single file's diff content — only materializes that one file.
    pub fn show_file(&self, rev: &str, path: &str) -> CoreResult<DiffHunk> {
        let (trees, _) = self.commit_trees(rev)?;
        self.diff_single_file(&trees, path)
    }

    /// Show diff for a renamed file: old content from `old_path` in parent tree,
    /// new content from `new_path` in commit tree.
    pub fn show_file_rename(
        &self,
        rev: &str,
        old_path: &str,
        new_path: &str,
    ) -> CoreResult<DiffHunk> {
        let (trees, _) = self.commit_trees(rev)?;
        let path_converter = self.path_converter();

        let old_repo_path = self.parse_named_diff_path("old", old_path)?;
        let new_repo_path = self.parse_named_diff_path("new", new_path)?;

        let old_matcher = FilesMatcher::new(std::iter::once(old_repo_path.as_ref()));
        let old_content =
            first_diff_content(&trees, &old_matcher)?.and_then(|(_, content)| content.old_content);

        let new_matcher = FilesMatcher::new(std::iter::once(new_repo_path.as_ref()));
        let new_content =
            first_diff_content(&trees, &new_matcher)?.and_then(|(_, content)| content.new_content);

        Ok(DiffHunk {
            path: path_converter.format_file_path(&new_repo_path),
            old_path: Some(path_converter.format_file_path(&old_repo_path)),
            old_content,
            new_content,
            hunk_type: HunkType::Renamed,
        })
    }

    /// Get insertions/deletions line count for a revision.
    pub fn diff_stats(&self, rev: &str) -> CoreResult<DiffStats> {
        let output = self.run_jj(&["diff", "--stat", "-r", rev])?;
        if let Some(summary) = output.lines().last() {
            let insertions = summary
                .split(',')
                .find(|s| s.contains("insertion"))
                .and_then(|s| s.trim().split_whitespace().next())
                .and_then(|n| n.parse::<u32>().ok())
                .unwrap_or(0);
            let deletions = summary
                .split(',')
                .find(|s| s.contains("deletion"))
                .and_then(|s| s.trim().split_whitespace().next())
                .and_then(|n| n.parse::<u32>().ok())
                .unwrap_or(0);
            Ok(DiffStats {
                insertions,
                deletions,
            })
        } else {
            Ok(DiffStats {
                insertions: 0,
                deletions: 0,
            })
        }
    }

    /// Fast: file list between two arbitrary revisions WITHOUT content.
    pub fn interdiff_summary(&self, from_rev: &str, to_rev: &str) -> CoreResult<ChangeDetail> {
        let (trees, info) = self.interdiff_trees(from_rev, to_rev)?;
        let diff = self.diff_file_list(&trees)?;
        Ok(ChangeDetail { info, diff })
    }

    /// Single file content between two arbitrary revisions.
    pub fn interdiff_file(&self, from_rev: &str, to_rev: &str, path: &str) -> CoreResult<DiffHunk> {
        let (trees, _) = self.interdiff_trees(from_rev, to_rev)?;
        self.diff_single_file(&trees, path)
    }
}
