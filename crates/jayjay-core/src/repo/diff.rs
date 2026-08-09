mod content;
mod current_text;
mod entry;
mod formats;
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
    repo: Arc<ReadonlyRepo>,
    before: MergedTree,
    after: MergedTree,
}

fn hunk_line_stats(hunk: &DiffHunk, ignore_whitespace: bool) -> FileDiffStats {
    let old = hunk.old.content.as_deref();
    let new = hunk.new.content.as_deref();
    // The canonical classifier covers every core placeholder (symlink, too-large, ...), unlike jj_diff's narrower display set.
    let editable = |text: Option<&str>| text.is_none_or(crate::placeholder::is_editable_text);
    let (insertions, deletions) = if editable(old) && editable(new) {
        jj_diff::count_changed_lines(
            old.unwrap_or_default(),
            new.unwrap_or_default(),
            ignore_whitespace,
        )
    } else {
        (0, 0)
    };
    FileDiffStats {
        path: hunk.path.clone(),
        insertions,
        deletions,
    }
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
        let mut diff = self.diff_file_list(&trees)?;
        if info.has_conflict {
            for (path, supported) in self.conflict_summaries(rev)? {
                if let Some(hunk) = diff.iter_mut().find(|hunk| hunk.path == path) {
                    hunk.supports_conflict_editor = supported;
                } else {
                    diff.push(conflict_summary_hunk(path, supported));
                }
            }
        }
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
        self.show_file_rename_with_mode(rev, old_path, new_path, DiffProjectionMode::Processed)
    }

    fn show_file_rename_with_mode(
        &self,
        rev: &str,
        old_path: &str,
        new_path: &str,
        projection_mode: DiffProjectionMode,
    ) -> CoreResult<DiffHunk> {
        let trees = self.commit_tree_pair(rev)?;

        let old_repo_path = self.parse_named_diff_path("old", old_path)?;
        let new_repo_path = self.parse_named_diff_path("new", new_path)?;

        let old_matcher = FilesMatcher::new(std::iter::once(old_repo_path.as_ref()));
        let old_diff = first_diff_content(&trees, &old_matcher, projection_mode)?;
        let (old, old_identity) = old_diff
            .map(|(_, content, identity)| (content.old, identity))
            .unwrap_or_default();

        let new_matcher = FilesMatcher::new(std::iter::once(new_repo_path.as_ref()));
        let new_diff = first_diff_content(&trees, &new_matcher, projection_mode)?;
        let (new, new_identity, projection, supports_file_editor) = new_diff
            .map(|(_, content, identity)| {
                (
                    content.new,
                    identity,
                    content.projection,
                    content.supports_file_editor,
                )
            })
            .unwrap_or_default();

        Ok(DiffHunk {
            path: new_repo_path.as_internal_file_string().to_owned(),
            old_path: Some(old_repo_path.as_internal_file_string().to_owned()),
            old,
            new,
            hunk_type: HunkType::Renamed,
            supports_conflict_editor: false,
            supports_file_editor,
            review_identity: hex_sha256(format!("rename|{old_identity}|{new_identity}").as_bytes()),
            projection,
        })
    }

    pub fn show_file_raw(&self, rev: &str, path: &str) -> CoreResult<DiffHunk> {
        let trees = self.commit_tree_pair(rev)?;
        self.diff_single_file_with_mode(&trees, path, DiffProjectionMode::Raw)
    }

    pub fn show_file_rename_raw(
        &self,
        rev: &str,
        old_path: &str,
        new_path: &str,
    ) -> CoreResult<DiffHunk> {
        self.show_file_rename_with_mode(rev, old_path, new_path, DiffProjectionMode::Raw)
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

    /// Per-file line counts for the revision, matching what Diff Edit presents: raw (unprojected) text, rename-aware, placeholder-only sides counted as zero.
    pub fn diff_file_stats(
        &self,
        rev: &str,
        ignore_whitespace: bool,
    ) -> CoreResult<Vec<FileDiffStats>> {
        let trees = self.commit_tree_pair(rev)?;
        self.diff_file_stats_walk(&trees, ignore_whitespace)
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

    pub fn interdiff_file_raw(
        &self,
        from_rev: &str,
        to_rev: &str,
        path: &str,
    ) -> CoreResult<DiffHunk> {
        let trees = self.interdiff_tree_pair(from_rev, to_rev)?;
        self.diff_single_file_with_mode(&trees, path, DiffProjectionMode::Raw)
    }
}

fn conflict_summary_hunk(path: String, supports_conflict_editor: bool) -> DiffHunk {
    DiffHunk {
        path,
        old_path: None,
        old: DiffContent::new(Some(String::new()), None),
        new: DiffContent::new(Some(String::new()), None),
        hunk_type: HunkType::Modified,
        supports_conflict_editor,
        supports_file_editor: false,
        review_identity: String::new(),
        projection: None,
    }
}
