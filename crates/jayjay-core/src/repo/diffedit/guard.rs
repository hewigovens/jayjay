use std::sync::Arc;

use jj_lib::merged_tree::MergedTree;
use jj_lib::repo::ReadonlyRepo;
use jj_lib::repo_path::RepoPath;

use crate::repo::Repo;
use crate::repo::support::block_on_result;
use crate::types::*;

impl Repo {
    /// Rejects selections whose rendered sides no longer match the source commit and its parent, so an intervening edit or rewrite (e.g. an editor save or a rebase after the diff rendered) can't be silently dropped or restored.
    pub(super) fn ensure_selections_are_current(
        &self,
        repo: &Arc<ReadonlyRepo>,
        tree: &MergedTree,
        parent_tree: &MergedTree,
        selections: &[DiffEditFileSelection],
    ) -> CoreResult<()> {
        for selection in selections {
            // Renames keep their old content under old_path; reject them up front so the old-side compare below can't misreport them as stale.
            if selection.hunk_type == HunkType::Renamed || selection.old_path.is_some() {
                return Err(CoreError::Internal {
                    message: format!("diff edit does not support renamed path {}", selection.path),
                });
            }
            let path = self.parse_repo_path(&selection.path)?;
            // An unresolved conflict materializes as marker text that can equal what the diff rendered, so a text compare alone would let partitioning write literal markers into the rewritten trees as resolved content.
            self.ensure_path_unconflicted(tree, path.as_ref(), &selection.path)?;
            self.ensure_path_unconflicted(parent_tree, path.as_ref(), &selection.path)?;
            let current = self.materialize_path_text(repo, tree, path.as_ref())?;
            if current != selection.new_content {
                return Err(CoreError::DiffSelectionStale {
                    path: selection.path.clone(),
                });
            }
            // partition_file_selection rebuilds unselected lines from old_content, so a parent rewritten after render is just as stale as a new-side edit.
            let parent = self.materialize_path_text(repo, parent_tree, path.as_ref())?;
            if parent != selection.old_content {
                return Err(CoreError::DiffSelectionStale {
                    path: selection.path.clone(),
                });
            }
        }
        Ok(())
    }

    fn ensure_path_unconflicted(
        &self,
        tree: &MergedTree,
        path: &RepoPath,
        display_path: &str,
    ) -> CoreResult<()> {
        let value = block_on_result(&format!("read {display_path}"), tree.path_value(path))?;
        if value.is_resolved() {
            Ok(())
        } else {
            Err(CoreError::Internal {
                message: format!("diff edit does not support conflicted file {display_path}"),
            })
        }
    }
}
