use std::collections::BTreeSet;
use std::sync::Arc;

use jj_lib::backend::TreeValue;
use jj_lib::commit::Commit;
use jj_lib::merge::{Merge, MergedTreeValue};
use jj_lib::merged_tree::MergedTree;
use jj_lib::merged_tree_builder::MergedTreeBuilder;
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::repo_path::RepoPath;
use jj_lib::rewrite::{CommitWithSelection, squash_commits};

use super::Repo;
use super::support::block_on_result;
use crate::diff::compute_file_diff_full;
use crate::types::*;

#[derive(Clone, Debug)]
struct RawLine {
    text: String,
    has_newline: bool,
}

#[derive(Debug)]
struct PartitionedSelection {
    selected_text: String,
    selected_exists: bool,
    remaining_text: String,
    remaining_exists: bool,
    selected_changed_lines: usize,
}

impl Repo {
    pub fn apply_diff_selection(
        &self,
        rev: &str,
        destination: DiffEditDestination,
        selections: &[DiffEditFileSelection],
        message: &str,
        ignore_whitespace: bool,
    ) -> CoreResult<()> {
        match destination {
            DiffEditDestination::RemoveFromSource => self.remove_diff_selection_from_source(
                rev,
                selections,
                ignore_whitespace,
            ),
            DiffEditDestination::MoveToWorkingCopy => {
                self.move_diff_selection_to_working_copy(rev, selections, ignore_whitespace)
            }
            DiffEditDestination::NewChild => {
                self.extract_diff_selection_as_new_child(
                    rev,
                    selections,
                    message,
                    ignore_whitespace,
                )
            }
            DiffEditDestination::NewParallel => {
                self.extract_diff_selection_as_parallel(
                    rev,
                    selections,
                    message,
                    ignore_whitespace,
                )
            }
        }
    }

    fn remove_diff_selection_from_source(
        &self,
        rev: &str,
        selections: &[DiffEditFileSelection],
        ignore_whitespace: bool,
    ) -> CoreResult<()> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;

        self.with_existing_commit_transaction(
            repo,
            commit,
            "remove selected changes",
            true,
            |repo, commit, repo_mut| {
                let parent_tree = self.load_parent_tree(repo, commit, "load parent tree")?;
                let remaining_tree = self.build_remaining_tree(
                    repo,
                    commit,
                    &parent_tree,
                    selections,
                    ignore_whitespace,
                )?;
                let write = repo_mut
                    .rewrite_commit(commit)
                    .set_tree(remaining_tree)
                    .write();
                block_on_result("rewrite source commit", write)?;
                Ok(())
            },
        )
    }

    fn move_diff_selection_to_working_copy(
        &self,
        rev: &str,
        selections: &[DiffEditFileSelection],
        ignore_whitespace: bool,
    ) -> CoreResult<()> {
        let repo = self.get_repo();
        let source = self.resolve_commit(&repo, rev)?;
        let destination = self.resolve_commit(&repo, "@")?;
        if source.id() == destination.id() {
            return Err(CoreError::Internal {
                message: "cannot move selected changes from @ to @".to_owned(),
            });
        }

        let mut tx = repo.start_transaction();
        let source_selection =
            self.build_commit_selection(&repo, &source, selections, ignore_whitespace)?;
        let squashed = block_on_result(
            "move selected changes to working copy",
            squash_commits(tx.repo_mut(), &[source_selection], &destination, true),
        )?;
        let Some(squashed) = squashed else {
            return Err(CoreError::Internal {
                message: "no changes selected".to_owned(),
            });
        };
        let write = squashed
            .commit_builder
            .set_description(destination.description())
            .write();
        block_on_result("write working-copy change", write)?;
        self.commit_transaction_rebase(tx, "move selected changes to working copy")
    }

    fn extract_diff_selection_as_new_child(
        &self,
        rev: &str,
        selections: &[DiffEditFileSelection],
        message: &str,
        ignore_whitespace: bool,
    ) -> CoreResult<()> {
        self.with_resolved_commit_transaction(
            rev,
            "extract selected changes as child",
            true,
            |repo, commit, repo_mut| {
                let source_selection =
                    self.build_commit_selection(repo, commit, selections, ignore_whitespace)?;
                let remaining_tree = self.build_remaining_tree(
                    repo,
                    commit,
                    &source_selection.parent_tree,
                    selections,
                    ignore_whitespace,
                )?;
                let rewritten_source = block_on_result(
                    "rewrite source commit",
                    repo_mut.rewrite_commit(commit).set_tree(remaining_tree).write(),
                )?;
                let child_tree = self.apply_selection_to_tree(
                    &source_selection,
                    rewritten_source.tree(),
                    "apply selected changes to child",
                )?;
                let child_description = self.diffedit_message(message, commit);
                let write = repo_mut
                    .new_commit(vec![rewritten_source.id().clone()], child_tree)
                    .set_description(&child_description)
                    .write();
                block_on_result("create child change", write)?;
                Ok(())
            },
        )
    }

    fn extract_diff_selection_as_parallel(
        &self,
        rev: &str,
        selections: &[DiffEditFileSelection],
        message: &str,
        ignore_whitespace: bool,
    ) -> CoreResult<()> {
        self.with_resolved_commit_transaction(
            rev,
            "extract selected changes as parallel",
            true,
            |repo, commit, repo_mut| {
                let source_selection =
                    self.build_commit_selection(repo, commit, selections, ignore_whitespace)?;
                let remaining_tree = self.build_remaining_tree(
                    repo,
                    commit,
                    &source_selection.parent_tree,
                    selections,
                    ignore_whitespace,
                )?;
                let write = repo_mut
                    .rewrite_commit(commit)
                    .set_tree(remaining_tree)
                    .write();
                block_on_result("rewrite source commit", write)?;
                let parallel_description = self.diffedit_message(message, commit);
                let write = repo_mut
                    .new_commit(commit.parent_ids().to_vec(), source_selection.selected_tree.clone())
                    .set_description(&parallel_description)
                    .write();
                block_on_result("create parallel change", write)?;
                Ok(())
            },
        )
    }

    fn build_commit_selection(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &Commit,
        selections: &[DiffEditFileSelection],
        ignore_whitespace: bool,
    ) -> CoreResult<CommitWithSelection> {
        let parent_tree = self.load_parent_tree(repo, commit, "load parent tree")?;
        let selected_tree =
            self.build_selected_tree(repo, commit, &parent_tree, selections, ignore_whitespace)?;
        Ok(CommitWithSelection {
            commit: commit.clone(),
            selected_tree,
            parent_tree,
        })
    }

    fn build_selected_tree(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &Commit,
        parent_tree: &MergedTree,
        selections: &[DiffEditFileSelection],
        ignore_whitespace: bool,
    ) -> CoreResult<MergedTree> {
        let source_tree = commit.tree();
        let mut builder = MergedTreeBuilder::new(parent_tree.clone());
        let mut selected_any = false;

        for selection in selections {
            let repo_path = self.parse_repo_path(&selection.path)?;
            let partition = self.partition_file_selection(selection, ignore_whitespace)?;
            if partition.selected_changed_lines == 0 {
                continue;
            }
            selected_any = true;

            if partition.selected_exists {
                let new_value = self.write_selected_file_value(
                    repo,
                    &source_tree,
                    parent_tree,
                    repo_path.as_ref(),
                    &partition.selected_text,
                )?;
                builder.set_or_remove(repo_path, new_value);
            } else {
                builder.set_or_remove(repo_path, Merge::absent());
            }
        }

        if !selected_any {
            return Err(CoreError::Internal {
                message: "no changes selected".to_owned(),
            });
        }

        block_on_result("write selected tree", builder.write_tree())
    }

    fn build_remaining_tree(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &Commit,
        parent_tree: &MergedTree,
        selections: &[DiffEditFileSelection],
        ignore_whitespace: bool,
    ) -> CoreResult<MergedTree> {
        let source_tree = commit.tree();
        let mut builder = MergedTreeBuilder::new(source_tree.clone());
        let mut selected_any = false;

        for selection in selections {
            let repo_path = self.parse_repo_path(&selection.path)?;
            let partition = self.partition_file_selection(selection, ignore_whitespace)?;
            if partition.selected_changed_lines == 0 {
                continue;
            }
            selected_any = true;

            if partition.remaining_exists {
                let new_value = self.write_selected_file_value(
                    repo,
                    &source_tree,
                    parent_tree,
                    repo_path.as_ref(),
                    &partition.remaining_text,
                )?;
                builder.set_or_remove(repo_path, new_value);
            } else {
                builder.set_or_remove(repo_path, Merge::absent());
            }
        }

        if !selected_any {
            return Err(CoreError::Internal {
                message: "no changes selected".to_owned(),
            });
        }

        block_on_result("write remaining tree", builder.write_tree())
    }

    fn partition_file_selection(
        &self,
        selection: &DiffEditFileSelection,
        ignore_whitespace: bool,
    ) -> CoreResult<PartitionedSelection> {
        partition_file_selection_impl(selection, ignore_whitespace)
    }

    fn write_selected_file_value(
        &self,
        repo: &Arc<ReadonlyRepo>,
        source_tree: &MergedTree,
        parent_tree: &MergedTree,
        path: &RepoPath,
        text: &str,
    ) -> CoreResult<MergedTreeValue> {
        let metadata = self
            .resolved_file_value(source_tree, path, "load selected file metadata")?
            .or_else(|| self.resolved_file_value(parent_tree, path, "load parent file metadata").ok().flatten())
            .ok_or_else(|| CoreError::Internal {
                message: format!("selected file metadata missing for {}", path.as_internal_file_string()),
            })?;

        let TreeValue::File {
            executable,
            copy_id,
            ..
        } = metadata
        else {
            return Err(CoreError::Internal {
                message: format!(
                    "diff edit only supports regular files: {}",
                    path.as_internal_file_string()
                ),
            });
        };

        let file_id = block_on_result(
            &format!("write file {}", path.as_internal_file_string()),
            repo.store().write_file(path, &mut text.as_bytes()),
        )?;
        Ok(Merge::normal(TreeValue::File {
            id: file_id,
            executable,
            copy_id,
        }))
    }

    fn resolved_file_value(
        &self,
        tree: &MergedTree,
        path: &RepoPath,
        context: &str,
    ) -> CoreResult<Option<TreeValue>> {
        let value = block_on_result(context, tree.path_value_async(path))?;
        value.into_resolved().map_err(|_| CoreError::Internal {
            message: format!("conflicted file values are not supported: {}", path.as_internal_file_string()),
        })
    }

    fn apply_selection_to_tree(
        &self,
        selection: &CommitWithSelection,
        base_tree: MergedTree,
        context: &str,
    ) -> CoreResult<MergedTree> {
        let selected_diff = block_on_result(
            "build selected diff",
            selection.diff_with_labels("source parent", "selected changes", "selected changes"),
        )?;
        block_on_result(
            context,
            MergedTree::merge(jj_lib::merge::Merge::from_diffs(
                (base_tree, "diff edit destination".to_owned()),
                [selected_diff],
            )),
        )
    }

    fn diffedit_message(&self, message: &str, commit: &Commit) -> String {
        let trimmed = message.trim();
        if !trimmed.is_empty() {
            trimmed.to_owned()
        } else {
            let description = commit.description().trim();
            if description.is_empty() {
                "selected changes".to_owned()
            } else {
                description.to_owned()
            }
        }
    }

    fn is_editable_text(text: &str) -> bool {
        !text.starts_with("<binary file")
            && !text.starts_with("<directory>")
            && !text.starts_with("<git submodule")
            && !text.starts_with("<conflict")
            && !text.starts_with("<access denied")
    }
}

fn partition_file_selection_impl(
    selection: &DiffEditFileSelection,
    ignore_whitespace: bool,
) -> CoreResult<PartitionedSelection> {
    if selection.hunk_type == HunkType::Renamed || selection.old_path.is_some() {
        return Err(CoreError::Internal {
            message: format!("diff edit does not support renamed path {}", selection.path),
        });
    }

    let old_text = selection.old_content.as_deref().unwrap_or_default();
    let new_text = selection.new_content.as_deref().unwrap_or_default();
    if !Repo::is_editable_text(old_text) || !Repo::is_editable_text(new_text) {
        return Err(CoreError::Internal {
            message: format!("diff edit only supports textual files: {}", selection.path),
        });
    }

    let old_lines = split_raw_lines(old_text);
    let new_lines = split_raw_lines(new_text);
    let diff = compute_file_diff_full(&selection.path, old_text, new_text, ignore_whitespace);
    let selected_indices = selected_line_indices(&selection.line_ranges);

    let mut selected_result = Vec::new();
    let mut remaining_result = Vec::new();
    let mut selected_changed_lines = 0usize;
    let mut total_changed_lines = 0usize;

    for (index, line) in diff.lines.iter().enumerate() {
        let is_selected = selected_indices.contains(&(index + 1));
        match line.style {
            crate::diff::DiffSpanStyle::Context | crate::diff::DiffSpanStyle::Unchanged => {
                if let Some(new_line_no) = line.new_line_no {
                    let cloned = clone_line(&new_lines, new_line_no)?;
                    selected_result.push(cloned.clone());
                    remaining_result.push(cloned);
                }
            }
            crate::diff::DiffSpanStyle::Removed => {
                total_changed_lines += 1;
                if is_selected {
                    selected_changed_lines += 1;
                    if let Some(old_line_no) = line.old_line_no {
                        remaining_result.push(clone_line(&old_lines, old_line_no)?);
                    }
                } else if let Some(old_line_no) = line.old_line_no {
                    selected_result.push(clone_line(&old_lines, old_line_no)?);
                }
            }
            crate::diff::DiffSpanStyle::Added => {
                total_changed_lines += 1;
                if is_selected {
                    selected_changed_lines += 1;
                    if let Some(new_line_no) = line.new_line_no {
                        selected_result.push(clone_line(&new_lines, new_line_no)?);
                    }
                } else if let Some(new_line_no) = line.new_line_no {
                    remaining_result.push(clone_line(&new_lines, new_line_no)?);
                }
            }
            crate::diff::DiffSpanStyle::Separator => {}
        }
    }

    let selected_exists = match selection.hunk_type {
        HunkType::Added => selected_changed_lines > 0,
        HunkType::Removed => selected_changed_lines < total_changed_lines,
        HunkType::Modified => selection.old_content.is_some(),
        HunkType::Renamed => false,
    };
    let remaining_exists = match selection.hunk_type {
        HunkType::Added => selected_changed_lines < total_changed_lines,
        HunkType::Removed => selected_changed_lines > 0,
        HunkType::Modified => selection.old_content.is_some(),
        HunkType::Renamed => false,
    };

    Ok(PartitionedSelection {
        selected_text: join_raw_lines(&selected_result),
        remaining_text: join_raw_lines(&remaining_result),
        selected_exists,
        remaining_exists,
        selected_changed_lines,
    })
}

fn split_raw_lines(text: &str) -> Vec<RawLine> {
    if text.is_empty() {
        return Vec::new();
    }

    text.split_inclusive('\n')
        .map(|segment| {
            let has_newline = segment.ends_with('\n');
            let text = if has_newline {
                segment[..segment.len() - 1].to_owned()
            } else {
                segment.to_owned()
            };
            RawLine { text, has_newline }
        })
        .collect()
}

fn join_raw_lines(lines: &[RawLine]) -> String {
    let mut result = String::new();
    for line in lines {
        result.push_str(&line.text);
        if line.has_newline {
            result.push('\n');
        }
    }
    result
}

fn clone_line(lines: &[RawLine], line_no: u32) -> CoreResult<RawLine> {
    lines
        .get((line_no.saturating_sub(1)) as usize)
        .cloned()
        .ok_or_else(|| CoreError::Internal {
            message: format!("missing line {line_no} in diff selection"),
        })
}

fn selected_line_indices(ranges: &[DiffEditRange]) -> BTreeSet<usize> {
    let mut indices = BTreeSet::new();
    for range in ranges {
        let start = range.start_line.min(range.end_line) as usize;
        let end = range.start_line.max(range.end_line) as usize;
        for index in start..=end {
            indices.insert(index);
        }
    }
    indices
}

#[cfg(test)]
mod tests {
    use super::*;

    fn partition(
        hunk_type: HunkType,
        old_content: Option<&str>,
        new_content: Option<&str>,
        ranges: &[(u32, u32)],
    ) -> PartitionedSelection {
        partition_file_selection_impl(
            &DiffEditFileSelection {
                path: "test.txt".to_owned(),
                old_path: None,
                old_content: old_content.map(str::to_owned),
                new_content: new_content.map(str::to_owned),
                hunk_type,
                line_ranges: ranges
                    .iter()
                    .map(|(start, end)| DiffEditRange {
                        start_line: *start,
                        end_line: *end,
                    })
                    .collect(),
            },
            false,
        )
        .expect("partition selection")
    }

    #[test]
    fn selecting_added_line_keeps_only_selected_change() {
        let selection = partition(
            HunkType::Modified,
            Some("a\nb\n"),
            Some("a\nx\n"),
            &[(3, 3)],
        );
        assert_eq!(selection.selected_text, "a\nb\nx\n");
        assert!(selection.selected_exists);
        assert_eq!(selection.selected_changed_lines, 1);
    }

    #[test]
    fn selecting_removed_and_added_lines_replaces_content() {
        let selection = partition(
            HunkType::Modified,
            Some("a\nb\n"),
            Some("a\nx\n"),
            &[(2, 3)],
        );
        assert_eq!(selection.selected_text, "a\nx\n");
        assert!(selection.selected_exists);
        assert_eq!(selection.selected_changed_lines, 2);
    }

    #[test]
    fn selecting_removed_line_on_deleted_file_produces_absent_selected_tree() {
        let selection = partition(HunkType::Removed, Some("a\n"), None, &[(1, 1)]);
        assert_eq!(selection.selected_text, "");
        assert!(!selection.selected_exists);
        assert_eq!(selection.remaining_text, "a\n");
        assert!(selection.remaining_exists);
    }

    #[test]
    fn selecting_part_of_added_file_keeps_partial_file() {
        let selection = partition(HunkType::Added, None, Some("a\nb\n"), &[(1, 1)]);
        assert_eq!(selection.selected_text, "a\n");
        assert!(selection.selected_exists);
        assert_eq!(selection.remaining_text, "b\n");
        assert!(selection.remaining_exists);
    }
}
