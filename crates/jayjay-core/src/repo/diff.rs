use std::path::PathBuf;
use std::sync::Arc;

use futures::StreamExt as _;
use jj_lib::commit::Commit as JjCommit;
use jj_lib::conflicts::{MaterializedTreeValue, materialize_tree_value};
use jj_lib::matchers::EverythingMatcher;
use jj_lib::merged_tree::TreeDiffEntry;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use pollster::FutureExt as _;

use super::Repo;
use crate::types::*;

impl Repo {
    pub fn show(&self, rev: &str) -> CoreResult<ChangeDetail> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        let info = self.commit_to_change_info(&repo, &commit);
        let diff = self.diff_hunks_for_commit(&repo, &commit)?;
        Ok(ChangeDetail { info, diff })
    }

    pub(crate) fn diff_hunks_for_commit(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &JjCommit,
    ) -> CoreResult<Vec<DiffHunk>> {
        let before_tree = commit.parent_tree(repo.as_ref()).block_on().map_err(|e| {
            CoreError::Internal {
                message: format!("load parent tree: {e}"),
            }
        })?;
        let after_tree = commit.tree();
        let path_converter = self.path_converter();
        let mut diff_stream = before_tree.diff_stream(&after_tree, &EverythingMatcher);
        let mut diff = Vec::new();

        while let Some(TreeDiffEntry { path, values }) = diff_stream.next().block_on() {
            let values = values.map_err(|e| CoreError::Internal {
                message: format!("tree diff {}: {e}", path.as_internal_file_string()),
            })?;
            let old_value = materialize_tree_value(
                repo.store(),
                &path,
                values.before,
                before_tree.labels(),
            )
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("materialize old {}: {e}", path.as_internal_file_string()),
            })?;
            let new_value = materialize_tree_value(
                repo.store(),
                &path,
                values.after,
                after_tree.labels(),
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
                path: PathBuf::from(path_converter.format_file_path(&path)),
                old_content: materialized_to_string(&path, old_value)?,
                new_content: materialized_to_string(&path, new_value)?,
                hunk_type,
            });
        }
        Ok(diff)
    }
}

fn materialized_to_string(
    path: &jj_lib::repo_path::RepoPath,
    value: MaterializedTreeValue,
) -> CoreResult<Option<String>> {
    match value {
        MaterializedTreeValue::Absent => Ok(None),
        MaterializedTreeValue::AccessDenied(err) => {
            Ok(Some(format!("<access denied: {err}>")))
        }
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
        MaterializedTreeValue::Symlink { target, .. } => {
            Ok(Some(format!("symlink -> {target}")))
        }
        MaterializedTreeValue::FileConflict(_) => Ok(Some("<conflicted file>".to_owned())),
        MaterializedTreeValue::OtherConflict { .. } => Ok(Some("<conflict>".to_owned())),
        MaterializedTreeValue::GitSubmodule(id) => {
            Ok(Some(format!("<git submodule {}>", id.hex())))
        }
        MaterializedTreeValue::Tree(_) => Ok(Some("<directory>".to_owned())),
    }
}
