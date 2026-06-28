mod content;
mod entry;
mod materialize;
mod rename;

use std::collections::HashSet;
use std::sync::Arc;

use jayjay_primitives::hex_sha256;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::matchers::FilesMatcher;
use jj_lib::merged_tree::MergedTree;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::ReadonlyRepo;

use crate::types::*;

use self::entry::first_diff_content;
use super::Repo;

pub(super) struct TreePair {
    pub(super) repo: Arc<ReadonlyRepo>,
    pub(super) before: MergedTree,
    pub(super) after: MergedTree,
}

impl Repo {
    fn commit_tree_pair(&self, rev: &str) -> CoreResult<TreePair> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        let before = self.load_parent_tree(&repo, &commit, "load parent tree")?;
        let after = commit.tree();
        Ok(TreePair {
            repo,
            before,
            after,
        })
    }

    fn commit_trees(&self, rev: &str) -> CoreResult<(TreePair, ChangeInfo)> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        let change_id = encode_reverse_hex(commit.change_id().as_bytes());
        let divergent_change_ids = if self.is_change_id_divergent(&repo, &change_id)? {
            HashSet::from([change_id])
        } else {
            HashSet::new()
        };
        let info = self.commit_to_change_info(&repo, &commit, None, Some(&divergent_change_ids));
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

    fn interdiff_tree_pair(&self, from_rev: &str, to_rev: &str) -> CoreResult<TreePair> {
        let repo = self.get_repo();
        let from_commit = self.resolve_commit(&repo, from_rev)?;
        let to_commit = self.resolve_commit(&repo, to_rev)?;
        let before = from_commit.tree();
        let after = to_commit.tree();
        Ok(TreePair {
            repo,
            before,
            after,
        })
    }

    fn interdiff_trees(&self, from_rev: &str, to_rev: &str) -> CoreResult<(TreePair, ChangeInfo)> {
        let repo = self.get_repo();
        let from_commit = self.resolve_commit(&repo, from_rev)?;
        let to_commit = self.resolve_commit(&repo, to_rev)?;
        let mut info = self.commit_to_change_info(&repo, &to_commit, None, None);
        // A divergent target must expose its commit id (not its ambiguous change id) as the selection revision, or later per-file content loads resolve the change id and fail.
        if self
            .is_change_id_divergent(&repo, &info.change_id)
            .unwrap_or(false)
        {
            info.is_divergent = true;
        }
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
        let trees = self.commit_tree_pair(rev)?;
        self.diff_single_file(&trees, path)
    }

    pub fn show_file_rename(
        &self,
        rev: &str,
        old_path: &str,
        new_path: &str,
    ) -> CoreResult<DiffHunk> {
        let trees = self.commit_tree_pair(rev)?;

        let old_repo_path = self.parse_named_diff_path("old", old_path)?;
        let new_repo_path = self.parse_named_diff_path("new", new_path)?;

        let old_matcher = FilesMatcher::new(std::iter::once(old_repo_path.as_ref()));
        let old_diff = first_diff_content(&trees, &old_matcher)?;
        let (old_content, old_preview, old_identity) = old_diff
            .map(|(_, content, identity)| (content.old_content, content.old_preview, identity))
            .unwrap_or((None, None, String::new()));

        let new_matcher = FilesMatcher::new(std::iter::once(new_repo_path.as_ref()));
        let new_diff = first_diff_content(&trees, &new_matcher)?;
        let (new_content, new_preview, new_identity) = new_diff
            .map(|(_, content, identity)| (content.new_content, content.new_preview, identity))
            .unwrap_or((None, None, String::new()));

        Ok(DiffHunk {
            path: new_repo_path.as_internal_file_string().to_owned(),
            old_path: Some(old_repo_path.as_internal_file_string().to_owned()),
            old_content,
            new_content,
            old_preview,
            new_preview,
            hunk_type: HunkType::Renamed,
            review_identity: hex_sha256(format!("rename|{old_identity}|{new_identity}").as_bytes()),
        })
    }

    pub fn diff_stats(&self, rev: &str) -> CoreResult<DiffStats> {
        let output = self.run_jj(&["--ignore-working-copy", "diff", "--stat", "-r", rev])?;
        // Summary line shape: "N files changed, I insertions(+), D deletions(-)".
        let field = |summary: &str, keyword: &str| -> u32 {
            summary
                .split(',')
                .find(|s| s.contains(keyword))
                .and_then(|s| s.split_whitespace().next())
                .and_then(|n| n.parse::<u32>().ok())
                .unwrap_or(0)
        };
        if let Some(summary) = output.lines().last() {
            Ok(DiffStats {
                files_changed: field(summary, "file"),
                insertions: field(summary, "insertion"),
                deletions: field(summary, "deletion"),
            })
        } else {
            Ok(DiffStats {
                files_changed: 0,
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

    pub fn interdiff_file(&self, from_rev: &str, to_rev: &str, path: &str) -> CoreResult<DiffHunk> {
        let trees = self.interdiff_tree_pair(from_rev, to_rev)?;
        self.diff_single_file(&trees, path)
    }
}
