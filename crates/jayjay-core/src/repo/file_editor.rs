use futures::AsyncReadExt as _;
use jj_lib::backend::TreeValue;
use jj_lib::commit::Commit;
use jj_lib::conflicts::{MaterializedTreeValue, materialize_tree_value};
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::merged_tree::MergedTree;
use jj_lib::merged_tree_builder::MergedTreeBuilder;
use jj_lib::object_id::ObjectId as _;
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::repo_path::RepoPathBuf;
use std::sync::Arc;

use super::Repo;
use super::support::block_on_result;
use crate::file_display::MAX_DIFF_BYTES;
use crate::types::*;

struct WorkingCopyFileTarget {
    repo: Arc<ReadonlyRepo>,
    commit: Commit,
    path: RepoPathBuf,
    tree: MergedTree,
}

impl Repo {
    /// Load an existing regular UTF-8 file from the current working-copy change.
    pub fn working_copy_file_editor(&self, path: &str) -> CoreResult<FileEditorData> {
        self.refresh_working_copy()?;
        let target = self.working_copy_file_target(path)?;
        let value = block_on_result(
            &format!("read working-copy file {path}"),
            target.tree.path_value(target.path.as_ref()),
        )?;
        let resolved = value.clone().into_resolved().map_err(|_| {
            CoreError::internal(format!(
                "{path}: conflicted files use the conflict resolver"
            ))
        })?;
        let Some(TreeValue::File { id, .. }) = resolved else {
            return Err(CoreError::internal(format!(
                "{path}: only existing regular files can be edited"
            )));
        };
        let materialized = block_on_result(
            &format!("materialize working-copy file {path}"),
            materialize_tree_value(
                target.repo.store(),
                target.path.as_ref(),
                value,
                target.tree.labels(),
            ),
        )?;
        let MaterializedTreeValue::File(file) = materialized else {
            return Err(CoreError::internal(format!(
                "{path}: only existing regular files can be edited"
            )));
        };
        let mut bytes = Vec::new();
        block_on_result(
            &format!("read working-copy file {path}"),
            file.reader
                .take(MAX_DIFF_BYTES as u64 + 1)
                .read_to_end(&mut bytes),
        )?;
        if bytes.len() > MAX_DIFF_BYTES {
            return Err(CoreError::internal(format!(
                "{path}: file is too large to edit"
            )));
        }
        if bytes.contains(&0) {
            return Err(CoreError::internal(format!(
                "{path}: binary files cannot be edited"
            )));
        }
        let content = String::from_utf8(bytes)
            .map_err(|_| CoreError::internal(format!("{path}: file is not valid UTF-8 text")))?;

        Ok(FileEditorData {
            path: path.to_owned(),
            change_id: encode_reverse_hex(target.commit.change_id().as_bytes()),
            file_id: id.hex(),
            content,
        })
    }

    /// Replace one working-copy file while preserving its executable and copy metadata.
    pub fn apply_working_copy_file_editor(
        &self,
        data: &FileEditorData,
        content: &str,
    ) -> CoreResult<()> {
        self.refresh_working_copy()?;
        let target = self.working_copy_file_target(&data.path)?;
        if encode_reverse_hex(target.commit.change_id().as_bytes()) != data.change_id {
            return Err(CoreError::FileEditorStale {
                path: data.path.clone(),
            });
        }
        self.ensure_commit_mutable(&target.repo, &target.commit, "@")?;

        let value = block_on_result(
            &format!("read working-copy file {}", data.path),
            target.tree.path_value(target.path.as_ref()),
        )?;
        let current = value
            .into_resolved()
            .map_err(|_| CoreError::FileEditorStale {
                path: data.path.clone(),
            })?;
        let Some(TreeValue::File {
            id,
            executable,
            copy_id,
        }) = current
        else {
            return Err(CoreError::FileEditorStale {
                path: data.path.clone(),
            });
        };
        if id.hex() != data.file_id {
            return Err(CoreError::FileEditorStale {
                path: data.path.clone(),
            });
        }

        let new_id = block_on_result(
            &format!("write working-copy file {}", data.path),
            target
                .repo
                .store()
                .write_file(target.path.as_ref(), &mut content.as_bytes()),
        )?;
        if new_id == id {
            return Ok(());
        }
        let mut builder = MergedTreeBuilder::new(target.tree);
        builder.set_or_remove(
            target.path,
            jj_lib::merge::Merge::normal(TreeValue::File {
                id: new_id,
                executable,
                copy_id,
            }),
        );
        let new_tree = block_on_result("write edited working-copy tree", builder.write_tree())?;
        self.rewrite_existing_commit_with_tree(
            target.repo,
            target.commit,
            "edit working-copy file in JayJay",
            true,
            "rewrite edited working-copy file",
            |_, _| Ok(new_tree),
        )
    }

    fn working_copy_file_target(&self, path: &str) -> CoreResult<WorkingCopyFileTarget> {
        let repo = self.get_repo();
        let commit = self.working_copy_commit(&repo)?;
        let path = self.parse_repo_path(path)?;
        let tree = commit.tree();
        Ok(WorkingCopyFileTarget {
            repo,
            commit,
            path,
            tree,
        })
    }
}

#[cfg(test)]
mod tests;
