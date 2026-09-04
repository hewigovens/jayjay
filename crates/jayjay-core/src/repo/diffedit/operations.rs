use std::sync::Arc;

use jj_lib::backend::MergedTreeValue;
use jj_lib::backend::TreeValue;
use jj_lib::commit::Commit;
use jj_lib::merge::Merge;
use jj_lib::merged_tree::MergedTree;
use jj_lib::merged_tree_builder::MergedTreeBuilder;
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::repo_path::RepoPath;
use jj_lib::rewrite::{CommitWithSelection, squash_commits};

use super::partition::partition_file_selection;
use crate::repo::Repo;
use crate::repo::support::block_on_result;
use crate::types::*;

impl Repo {
    pub fn apply_diff_selection(
        &self,
        rev: &str,
        destination: DiffEditDestination,
        selections: &[DiffEditFileSelection],
        message: &str,
        ignore_whitespace: bool,
    ) -> CoreResult<()> {
        // Snapshot disk edits first so the post-rewrite checkout can't clobber un-snapshotted edits.
        self.refresh_working_copy()?;
        match destination {
            DiffEditDestination::RemoveFromSource => {
                self.remove_diff_selection_from_source(rev, selections, ignore_whitespace)
            }
            DiffEditDestination::MoveToWorkingCopy => {
                self.move_diff_selection_to_working_copy(rev, selections, ignore_whitespace)
            }
            DiffEditDestination::NewChild => self.extract_diff_selection_as_new_child(
                rev,
                selections,
                message,
                ignore_whitespace,
            ),
            DiffEditDestination::NewParallel => {
                self.extract_diff_selection_as_parallel(rev, selections, message, ignore_whitespace)
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
        self.ensure_commit_mutable(&repo, &commit, rev)?;
        let parent_tree = self.load_parent_tree(&repo, &commit, "load parent tree")?;
        self.ensure_selections_are_current(&repo, &commit.tree(), &parent_tree, selections)?;

        self.with_existing_commit_transaction(
            repo,
            commit,
            "remove selected changes",
            true,
            |repo, commit, repo_mut| {
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
        self.ensure_commit_mutable(&repo, &source, rev)?;
        let destination = self.resolve_commit(&repo, "@")?;
        if source.id() == destination.id() {
            return Err(CoreError::Internal {
                message: "cannot move selected changes from @ to @".to_owned(),
            });
        }
        let parent_tree = self.load_parent_tree(&repo, &source, "load parent tree")?;
        self.ensure_selections_are_current(&repo, &source.tree(), &parent_tree, selections)?;

        let mut tx = repo.start_transaction();
        let source_selection = self.build_commit_selection(
            &repo,
            &source,
            parent_tree,
            selections,
            ignore_whitespace,
        )?;
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
                self.ensure_commit_mutable(repo, commit, rev)?;
                let parent_tree = self.load_parent_tree(repo, commit, "load parent tree")?;
                self.ensure_selections_are_current(repo, &commit.tree(), &parent_tree, selections)?;
                let source_selection = self.build_commit_selection(
                    repo,
                    commit,
                    parent_tree,
                    selections,
                    ignore_whitespace,
                )?;
                let remaining_tree = self.build_remaining_tree(
                    repo,
                    commit,
                    &source_selection.parent_tree,
                    selections,
                    ignore_whitespace,
                )?;
                let rewritten_source = block_on_result(
                    "rewrite source commit",
                    repo_mut
                        .rewrite_commit(commit)
                        .set_tree(remaining_tree)
                        .write(),
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
                self.ensure_commit_mutable(repo, commit, rev)?;
                let parent_tree = self.load_parent_tree(repo, commit, "load parent tree")?;
                self.ensure_selections_are_current(repo, &commit.tree(), &parent_tree, selections)?;
                let source_selection = self.build_commit_selection(
                    repo,
                    commit,
                    parent_tree,
                    selections,
                    ignore_whitespace,
                )?;
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
                    .new_commit(
                        commit.parent_ids().to_vec(),
                        source_selection.selected_tree.clone(),
                    )
                    .set_description(&parallel_description)
                    .write();
                block_on_result("create parallel change", write)?;
                Ok(())
            },
        )
    }

    // Takes the parent tree the caller already ran the staleness guard against, so validation and rewrite use the same base.
    fn build_commit_selection(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &Commit,
        parent_tree: MergedTree,
        selections: &[DiffEditFileSelection],
        ignore_whitespace: bool,
    ) -> CoreResult<CommitWithSelection> {
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
            let partition = partition_file_selection(selection, ignore_whitespace)?;
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
            let partition = partition_file_selection(selection, ignore_whitespace)?;
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
            .or_else(|| {
                self.resolved_file_value(parent_tree, path, "load parent file metadata")
                    .ok()
                    .flatten()
            })
            .ok_or_else(|| CoreError::Internal {
                message: format!(
                    "selected file metadata missing for {}",
                    path.as_internal_file_string()
                ),
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
        let value = block_on_result(context, tree.path_value(path))?;
        value.into_resolved().map_err(|_| CoreError::Internal {
            message: format!(
                "conflicted file values are not supported: {}",
                path.as_internal_file_string()
            ),
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
}
