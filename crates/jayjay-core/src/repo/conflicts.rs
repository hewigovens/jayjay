use std::sync::Arc;

use jj_lib::backend::TreeValue;
use jj_lib::commit::Commit;
use jj_lib::conflicts::{
    MaterializedTreeValue, choose_materialized_conflict_marker_len,
    materialize_merge_result_to_bytes, materialize_tree_value, update_from_content,
};
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::merged_tree_builder::MergedTreeBuilder;
use jj_lib::object_id::ObjectId as _;
use jj_lib::repo::{ReadonlyRepo, Repo as _};

use super::Repo;
use crate::file_display::{MAX_DIFF_BYTES, bytes_to_display};
use crate::repo::support::block_on_result;
use crate::types::*;

pub(super) use self::content::materialized_conflict_supports_editor;
use self::content::{
    conflict_fingerprint, conflict_materialize_options, conflict_supports_editor,
    conflict_total_bytes, display_conflict_term,
};

mod content;

impl Repo {
    /// List conflicted files for a revision.
    pub fn resolve_list(&self, rev: &str) -> CoreResult<Vec<String>> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        commit
            .tree()
            .conflicts()
            .map(|(path, value)| {
                value.map_err(|error| CoreError::Internal {
                    message: format!("read conflict {}: {error}", path.as_internal_file_string()),
                })?;
                Ok(path.as_internal_file_string().to_owned())
            })
            .collect()
    }

    pub(super) fn conflict_summaries(&self, rev: &str) -> CoreResult<Vec<(String, bool)>> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        let tree = commit.tree();
        tree.conflicts()
            .map(|(path, value)| {
                let value = value.map_err(|error| CoreError::Internal {
                    message: format!("read conflict {}: {error}", path.as_internal_file_string()),
                })?;
                let path_string = path.as_internal_file_string().to_owned();
                let supported = conflict_supports_editor(repo.store(), path.as_ref(), &value)?;
                Ok((path_string, supported))
            })
            .collect()
    }

    /// Resolve a conflicted file using a named tool (e.g. ":ours", ":theirs", or an editor).
    pub fn resolve_with_tool(&self, rev: &str, path: &str, tool: &str) -> CoreResult<()> {
        self.run_jj_reload(&["resolve", "-r", rev, "--tool", tool, path])
    }

    /// Load a file conflict for editing inside the current repository window.
    pub fn conflict_editor(&self, rev: &str, path: &str) -> CoreResult<ConflictEditorData> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        let is_working_copy = self.is_working_copy_commit(&repo, &commit);
        // Only editing the working-copy change warrants a snapshot; loading an ancestor's conflict must not create operations.
        let (repo, commit) = if is_working_copy {
            self.refresh_working_copy()?;
            let repo = self.get_repo();
            let commit = self.working_copy_commit(&repo)?;
            (repo, commit)
        } else {
            (repo, commit)
        };
        let repo_path = self.parse_repo_path(path)?;
        let tree = commit.tree();
        let value = block_on_result(
            &format!("read conflict {path}"),
            tree.path_value(repo_path.as_ref()),
        )?;
        let materialized = block_on_result(
            &format!("materialize conflict {path}"),
            materialize_tree_value(repo.store(), repo_path.as_ref(), value, tree.labels()),
        )?;
        let is_text = materialized_conflict_supports_editor(&materialized);
        let MaterializedTreeValue::FileConflict(file) = materialized else {
            return Err(CoreError::Internal {
                message: format!("{path} is not an editable file conflict"),
            });
        };

        let marker_length = choose_materialized_conflict_marker_len(&file.contents);
        let total_bytes = conflict_total_bytes(&file.contents);
        let options = conflict_materialize_options(marker_length);
        let result = if total_bytes > MAX_DIFF_BYTES {
            format!("<conflict too large to edit (over {MAX_DIFF_BYTES} bytes)>")
        } else {
            bytes_to_display(&materialize_merge_result_to_bytes(
                &file.contents,
                &file.labels,
                &options,
            ))
        };
        let hunks = if is_text {
            crate::merge_editor::merge_editor_hunks(&file.contents, &options, &result)
        } else {
            Vec::new()
        };

        Ok(ConflictEditorData {
            path: path.to_owned(),
            is_working_copy,
            change_id: encode_reverse_hex(commit.change_id().as_bytes()),
            conflict_id: conflict_fingerprint(&file.unsimplified_ids),
            left: display_conflict_term(file.contents.get_add(0), total_bytes),
            base: display_conflict_term(file.contents.get_remove(0), total_bytes),
            right: display_conflict_term(file.contents.get_add(1), total_bytes),
            result,
            marker_length: marker_length as u32,
            side_count: file.contents.num_sides() as u32,
            is_text,
            hunks,
        })
    }

    /// Parse the edited marker text with jj's own conflict parser and rewrite the selected change.
    pub fn apply_conflict_editor(
        &self,
        rev: &str,
        data: &ConflictEditorData,
        content: &str,
    ) -> CoreResult<()> {
        let path = data.path.as_str();
        if !data.is_text {
            return Err(CoreError::internal(format!(
                "{path} is not an editable text conflict"
            )));
        }
        self.refresh_working_copy()?;
        let repo = self.get_repo();
        let commit = if data.is_working_copy {
            self.working_copy_commit(&repo)?
        } else {
            self.resolve_commit(&repo, rev)?
        };
        if encode_reverse_hex(commit.change_id().as_bytes()) != data.change_id {
            return Err(CoreError::ConflictEditorStale {
                path: path.to_owned(),
            });
        }
        self.ensure_commit_mutable(&repo, &commit, rev)?;

        let repo_path = self.parse_repo_path(path)?;
        let tree = commit.tree();
        let current_value = block_on_result(
            &format!("read conflict {path}"),
            tree.path_value(repo_path.as_ref()),
        )?;
        let materialized = block_on_result(
            &format!("materialize conflict {path}"),
            materialize_tree_value(
                repo.store(),
                repo_path.as_ref(),
                current_value.clone(),
                tree.labels(),
            ),
        )?;
        let MaterializedTreeValue::FileConflict(file) = materialized else {
            return Err(CoreError::ConflictEditorStale {
                path: path.to_owned(),
            });
        };
        // If the sides changed since load, applying the stale marker text would silently discard the incoming side.
        if conflict_fingerprint(&file.unsimplified_ids) != data.conflict_id {
            return Err(CoreError::ConflictEditorStale {
                path: path.to_owned(),
            });
        }
        let new_file_ids = block_on_result(
            &format!("update conflict {path}"),
            update_from_content(
                &file.unsimplified_ids,
                repo.store(),
                repo_path.as_ref(),
                content.as_bytes(),
                data.marker_length as usize,
            ),
        )?;
        let new_value = match new_file_ids.into_resolved() {
            Ok(Some(id)) => {
                if let (Some(executable), Some(copy_id)) = (file.executable, file.copy_id.clone()) {
                    jj_lib::merge::Merge::normal(TreeValue::File {
                        id,
                        executable,
                        copy_id,
                    })
                } else {
                    let expanded_ids = file
                        .unsimplified_ids
                        .map(|old_id| old_id.as_ref().map(|_| id.clone()));
                    current_value.with_new_file_ids(&expanded_ids)
                }
            }
            Ok(None) => jj_lib::merge::Merge::absent(),
            Err(new_file_ids) => current_value.with_new_file_ids(&new_file_ids),
        };
        let mut builder = MergedTreeBuilder::new(tree);
        builder.set_or_remove(repo_path, new_value);
        let new_tree = block_on_result("write conflict resolution", builder.write_tree())?;
        self.rewrite_existing_commit_with_tree(
            repo,
            commit,
            "resolve conflict in JayJay",
            true,
            "rewrite conflict resolution",
            |_, _| Ok(new_tree),
        )
    }

    fn is_working_copy_commit(&self, repo: &Arc<ReadonlyRepo>, commit: &Commit) -> bool {
        repo.view().get_wc_commit_id(self.workspace_name.as_ref()) == Some(commit.id())
    }

    /// Resolve a file by accepting "ours" (side #1).
    pub fn resolve_use_ours(&self, rev: &str, path: &str) -> CoreResult<()> {
        self.resolve_with_tool(rev, path, ":ours")
    }

    /// Resolve a file by accepting "theirs" (side #2).
    pub fn resolve_use_theirs(&self, rev: &str, path: &str) -> CoreResult<()> {
        self.resolve_with_tool(rev, path, ":theirs")
    }

    /// Read a file's content (including conflict markers) from a revision.
    pub fn file_content(&self, rev: &str, path: &str) -> CoreResult<String> {
        self.run_jj(&["file", "show", "-r", rev, path])
    }
}

#[cfg(test)]
mod tests;
