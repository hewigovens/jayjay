use std::path::Path;
use std::sync::Arc;

use futures::StreamExt as _;
use jj_lib::conflicts::{MaterializedTreeValue, materialize_tree_value};
use jj_lib::matchers::EverythingMatcher;
use jj_lib::merged_tree::{MergedTree, TreeDiffEntry};
use jj_lib::object_id::ObjectId;
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use pollster::FutureExt as _;

use super::Repo;
use crate::types::*;

/// Resolved pair of trees for diffing.
struct TreePair {
    repo: Arc<ReadonlyRepo>,
    before: MergedTree,
    after: MergedTree,
}

impl Repo {
    /// Resolve a commit to its parent→commit tree pair.
    fn commit_trees(&self, rev: &str) -> CoreResult<(TreePair, ChangeInfo)> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        let info = self.commit_to_change_info(&repo, &commit);
        let before =
            commit
                .parent_tree(repo.as_ref())
                .block_on()
                .map_err(|e| CoreError::Internal {
                    message: format!("load parent tree: {e}"),
                })?;
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

    /// Walk tree diff and return file list WITHOUT content (fast).
    fn diff_file_list(&self, trees: &TreePair) -> CoreResult<Vec<DiffHunk>> {
        let path_converter = self.path_converter();
        let mut diff_stream = trees.before.diff_stream(&trees.after, &EverythingMatcher);
        let mut files = Vec::new();

        while let Some(TreeDiffEntry { path, values }) = diff_stream.next().block_on() {
            let values = values.map_err(|e| CoreError::Internal {
                message: format!("tree diff {}: {e}", path.as_internal_file_string()),
            })?;
            let hunk_type = match (values.before.is_absent(), values.after.is_absent()) {
                (true, false) => HunkType::Added,
                (false, true) => HunkType::Removed,
                _ => HunkType::Modified,
            };
            files.push(DiffHunk {
                path: path_converter.format_file_path(&path),
                old_path: None,
                old_content: None,
                new_content: None,
                hunk_type,
            });
        }
        detect_renames(&mut files);
        Ok(files)
    }

    /// Walk tree diff and return all hunks WITH content.
    fn diff_all_files(&self, trees: &TreePair) -> CoreResult<Vec<DiffHunk>> {
        let path_converter = self.path_converter();
        let mut diff_stream = trees.before.diff_stream(&trees.after, &EverythingMatcher);
        let mut diff = Vec::new();

        while let Some(TreeDiffEntry { path, values }) = diff_stream.next().block_on() {
            let values = values.map_err(|e| CoreError::Internal {
                message: format!("tree diff {}: {e}", path.as_internal_file_string()),
            })?;
            let old_value = materialize_tree_value(
                trees.repo.store(),
                &path,
                values.before,
                trees.before.labels(),
            )
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("materialize old {}: {e}", path.as_internal_file_string()),
            })?;
            let new_value = materialize_tree_value(
                trees.repo.store(),
                &path,
                values.after,
                trees.after.labels(),
            )
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("materialize new {}: {e}", path.as_internal_file_string()),
            })?;

            let hunk_type = match (old_value.is_absent(), new_value.is_absent()) {
                (true, false) => HunkType::Added,
                (false, true) => HunkType::Removed,
                _ => HunkType::Modified,
            };
            diff.push(DiffHunk {
                path: path_converter.format_file_path(&path),
                old_path: None,
                old_content: materialized_to_string(&path, old_value)?,
                new_content: materialized_to_string(&path, new_value)?,
                hunk_type,
            });
        }
        detect_renames(&mut diff);
        Ok(diff)
    }

    /// Materialize a single file between two trees.
    fn diff_single_file(&self, trees: &TreePair, path: &str) -> CoreResult<DiffHunk> {
        let path_converter = self.path_converter();
        let repo_path = jj_lib::repo_path::RepoPathBuf::parse_fs_path(&self.path, &self.path, path)
            .map_err(|e| CoreError::Internal {
                message: format!("invalid path: {e}"),
            })?;
        let matcher = jj_lib::matchers::FilesMatcher::new(std::iter::once(repo_path.as_ref()));
        let mut diff_stream = trees.before.diff_stream(&trees.after, &matcher);

        if let Some(TreeDiffEntry {
            path: entry_path,
            values,
        }) = diff_stream.next().block_on()
        {
            let values = values.map_err(|e| CoreError::Internal {
                message: format!("tree diff: {e}"),
            })?;
            let old_value = materialize_tree_value(
                trees.repo.store(),
                &entry_path,
                values.before,
                trees.before.labels(),
            )
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("materialize old: {e}"),
            })?;
            let new_value = materialize_tree_value(
                trees.repo.store(),
                &entry_path,
                values.after,
                trees.after.labels(),
            )
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("materialize new: {e}"),
            })?;

            let hunk_type = match (old_value.is_absent(), new_value.is_absent()) {
                (true, false) => HunkType::Added,
                (false, true) => HunkType::Removed,
                _ => HunkType::Modified,
            };
            Ok(DiffHunk {
                path: path_converter.format_file_path(&entry_path),
                old_path: None,
                old_content: materialized_to_string(&entry_path, old_value)?,
                new_content: materialized_to_string(&entry_path, new_value)?,
                hunk_type,
            })
        } else {
            Err(CoreError::Internal {
                message: format!("file not found in diff: {path}"),
            })
        }
    }

    // -- Public API: single-revision diff --

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

        let old_repo_path =
            jj_lib::repo_path::RepoPathBuf::parse_fs_path(&self.path, &self.path, old_path)
                .map_err(|e| CoreError::Internal {
                    message: format!("invalid old path: {e}"),
                })?;
        let new_repo_path =
            jj_lib::repo_path::RepoPathBuf::parse_fs_path(&self.path, &self.path, new_path)
                .map_err(|e| CoreError::Internal {
                    message: format!("invalid new path: {e}"),
                })?;

        // Materialize old content from parent tree at old_path
        let old_matcher =
            jj_lib::matchers::FilesMatcher::new(std::iter::once(old_repo_path.as_ref()));
        let mut old_stream = trees.before.diff_stream(&trees.after, &old_matcher);
        let old_content = if let Some(TreeDiffEntry { path, values }) = old_stream.next().block_on()
        {
            let values = values.map_err(|e| CoreError::Internal {
                message: format!("tree diff old: {e}"),
            })?;
            let old_value = materialize_tree_value(
                trees.repo.store(),
                &path,
                values.before,
                trees.before.labels(),
            )
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("materialize old: {e}"),
            })?;
            materialized_to_string(&path, old_value)?
        } else {
            None
        };

        // Materialize new content from commit tree at new_path
        let new_matcher =
            jj_lib::matchers::FilesMatcher::new(std::iter::once(new_repo_path.as_ref()));
        let mut new_stream = trees.before.diff_stream(&trees.after, &new_matcher);
        let new_content = if let Some(TreeDiffEntry { path, values }) = new_stream.next().block_on()
        {
            let values = values.map_err(|e| CoreError::Internal {
                message: format!("tree diff new: {e}"),
            })?;
            let new_value = materialize_tree_value(
                trees.repo.store(),
                &path,
                values.after,
                trees.after.labels(),
            )
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("materialize new: {e}"),
            })?;
            materialized_to_string(&path, new_value)?
        } else {
            None
        };

        Ok(DiffHunk {
            path: path_converter.format_file_path(&new_repo_path),
            old_path: Some(path_converter.format_file_path(&old_repo_path)),
            old_content,
            new_content,
            hunk_type: HunkType::Renamed,
        })
    }

    // -- Public API: interdiff (two arbitrary revisions) --

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

/// Detect renames by matching removed+added files via content similarity or filename similarity.
fn detect_renames(hunks: &mut Vec<DiffHunk>) {
    let removed_indices: Vec<usize> = hunks
        .iter()
        .enumerate()
        .filter(|(_, h)| h.hunk_type == HunkType::Removed)
        .map(|(i, _)| i)
        .collect();
    let added_indices: Vec<usize> = hunks
        .iter()
        .enumerate()
        .filter(|(_, h)| h.hunk_type == HunkType::Added)
        .map(|(i, _)| i)
        .collect();

    let mut matched_removed = Vec::new();
    let mut matched_added = Vec::new();

    for &ri in &removed_indices {
        let mut best_match: Option<(usize, f64)> = None;

        for &ai in &added_indices {
            if matched_added.contains(&ai) {
                continue;
            }
            let score = rename_score(&hunks[ri], &hunks[ai]);
            if score > 0.5 && !best_match.is_some_and(|(_, s)| score <= s) {
                best_match = Some((ai, score));
            }
        }

        if let Some((ai, score)) = best_match {
            let old_path = hunks[ri].path.clone();
            hunks[ai].old_path = Some(old_path);
            hunks[ai].hunk_type = HunkType::Renamed;

            // If content is identical, clear both sides (pure rename)
            if score >= 1.0 {
                hunks[ai].old_content = None;
                hunks[ai].new_content = None;
            } else {
                // Rename + modify: keep content for diff display
                hunks[ai].old_content = hunks[ri].old_content.clone();
            }

            matched_removed.push(ri);
            matched_added.push(ai);
        }
    }

    matched_removed.sort_unstable();
    for &i in matched_removed.iter().rev() {
        hunks.remove(i);
    }
}

/// Score how likely a removed+added pair is a rename. Returns 0.0–1.0.
fn rename_score(removed: &DiffHunk, added: &DiffHunk) -> f64 {
    let old_path = Path::new(&removed.path);
    let new_path = Path::new(&added.path);
    let old_name = old_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let new_name = new_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    let old_content = removed.old_content.as_deref().unwrap_or("");
    let new_content = added.new_content.as_deref().unwrap_or("");

    // Exact content match → definite rename
    if !old_content.is_empty() && old_content == new_content {
        return 1.0;
    }

    // Same filename (case-insensitive) → likely rename (e.g., Justfile → justfile)
    if !old_name.is_empty() && old_name.eq_ignore_ascii_case(new_name) {
        // Same name different case, or same name different directory
        let content_sim = content_similarity(old_content, new_content);
        // Even with content changes, same filename is a strong signal
        return 0.6 + content_sim * 0.4;
    }

    // Same extension + high content similarity → probable rename
    let old_ext = old_path.extension().and_then(|e| e.to_str());
    let new_ext = new_path.extension().and_then(|e| e.to_str());
    if old_ext == new_ext && old_ext.is_some() {
        let sim = content_similarity(old_content, new_content);
        if sim > 0.7 {
            return sim;
        }
    }

    0.0
}

/// Rough content similarity: ratio of matching lines.
fn content_similarity(a: &str, b: &str) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let a_lines: std::collections::HashSet<&str> = a.lines().collect();
    let b_lines: std::collections::HashSet<&str> = b.lines().collect();
    let intersection = a_lines.intersection(&b_lines).count();
    let union = a_lines.union(&b_lines).count();
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

fn materialized_to_string(
    path: &jj_lib::repo_path::RepoPath,
    value: MaterializedTreeValue,
) -> CoreResult<Option<String>> {
    match value {
        MaterializedTreeValue::Absent => Ok(None),
        MaterializedTreeValue::AccessDenied(err) => Ok(Some(format!("<access denied: {err}>"))),
        MaterializedTreeValue::File(mut file) => {
            let bytes = file
                .read_all(path)
                .block_on()
                .map_err(|e| CoreError::Internal {
                    message: format!("read file {}: {e}", path.as_internal_file_string()),
                })?;
            if bytes.contains(&0) {
                return Ok(Some(format!("<binary file ({} bytes)>", bytes.len())));
            }
            match String::from_utf8(bytes) {
                Ok(text) => Ok(Some(text)),
                Err(err) => Ok(Some(format!(
                    "<binary file ({} bytes)>",
                    err.into_bytes().len()
                ))),
            }
        }
        MaterializedTreeValue::Symlink { target, .. } => Ok(Some(format!("symlink -> {target}"))),
        MaterializedTreeValue::FileConflict(_) => Ok(Some("<conflicted file>".to_owned())),
        MaterializedTreeValue::OtherConflict { .. } => Ok(Some("<conflict>".to_owned())),
        MaterializedTreeValue::GitSubmodule(id) => {
            Ok(Some(format!("<git submodule {}>", id.hex())))
        }
        MaterializedTreeValue::Tree(_) => Ok(Some("<directory>".to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn hunk(path: &str, hunk_type: HunkType, old: Option<&str>, new: Option<&str>) -> DiffHunk {
        DiffHunk {
            path: path.to_owned(),
            old_path: None,
            old_content: old.map(|s| s.to_owned()),
            new_content: new.map(|s| s.to_owned()),
            hunk_type,
        }
    }

    #[test]
    fn content_similarity_both_empty_is_identical() {
        assert_eq!(content_similarity("", ""), 1.0);
    }

    #[test]
    fn content_similarity_identical() {
        assert_eq!(content_similarity("a\nb\n", "a\nb\n"), 1.0);
    }

    #[test]
    fn content_similarity_disjoint() {
        assert_eq!(content_similarity("a\n", "z\n"), 0.0);
    }

    #[test]
    fn rename_detected_with_content() {
        let mut hunks = vec![
            hunk("old.rs", HunkType::Removed, Some("fn main() {}"), None),
            hunk("new.rs", HunkType::Added, None, Some("fn main() {}")),
        ];
        detect_renames(&mut hunks);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].hunk_type, HunkType::Renamed);
        assert_eq!(hunks[0].path, "new.rs");
        assert_eq!(hunks[0].old_path.as_deref(), Some("old.rs"));
    }

    #[test]
    fn rename_detected_same_extension_no_content() {
        // In summary mode (no content), same-extension files are matched as renames
        // (content_similarity("","") = 1.0 — treated as identical).
        // Swift lazy-loads actual content for display.
        let mut hunks = vec![
            hunk("PLAN.md", HunkType::Removed, None, None),
            hunk("Roadmap.md", HunkType::Added, None, None),
        ];
        detect_renames(&mut hunks);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].hunk_type, HunkType::Renamed);
        assert_eq!(hunks[0].old_path.as_deref(), Some("PLAN.md"));
    }

    #[test]
    fn rename_same_filename_different_dir_no_content() {
        let mut hunks = vec![
            hunk("src/lib.rs", HunkType::Removed, None, None),
            hunk("core/lib.rs", HunkType::Added, None, None),
        ];
        detect_renames(&mut hunks);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].hunk_type, HunkType::Renamed);
    }

    #[test]
    fn no_rename_across_different_extensions() {
        let mut hunks = vec![
            hunk("old.rs", HunkType::Removed, None, None),
            hunk("new.py", HunkType::Added, None, None),
        ];
        detect_renames(&mut hunks);
        assert_eq!(hunks.len(), 2, "different extensions should not match");
    }
}
