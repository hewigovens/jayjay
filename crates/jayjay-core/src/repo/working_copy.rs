use jj_lib::matchers::{EverythingMatcher, NothingMatcher};
use jj_lib::repo::Repo as _;
use jj_lib::working_copy::SnapshotOptions;

use super::Repo;
use super::support::{block_on_result, load_repo_at_head, load_workspace_internal};
use super::working_copy_ignore::{WorkingCopyIgnoreMatcher, base_git_ignores};
use crate::types::*;

impl Repo {
    pub(crate) fn check_out_current_working_copy(&self, context: &str) -> CoreResult<()> {
        let mut workspace = load_workspace_internal(&self.path, context)?;
        let repo = load_repo_at_head(&workspace, context)?;
        let wc_commit_id = repo
            .view()
            .get_wc_commit_id(self.workspace_name.as_ref())
            .ok_or_else(|| CoreError::Internal {
                message: format!(
                    "workspace {} has no working-copy commit",
                    self.workspace_name.as_symbol()
                ),
            })?
            .clone();
        let wc_commit =
            repo.store()
                .get_commit(&wc_commit_id)
                .map_err(|e| CoreError::Internal {
                    message: format!("load working-copy commit: {e}"),
                })?;
        block_on_result(
            context,
            workspace.check_out(repo.op_id().clone(), None, &wc_commit),
        )?;
        self.set_repo(repo);
        Ok(())
    }

    pub fn refresh_working_copy(&self) -> CoreResult<()> {
        let mut workspace = load_workspace_internal(&self.path, "load workspace for snapshot")?;

        let repo = load_repo_at_head(&workspace, "load repo for snapshot")?;

        let wc_commit_id = repo
            .view()
            .get_wc_commit_id(self.workspace_name.as_ref())
            .ok_or_else(|| CoreError::Internal {
                message: format!(
                    "workspace {} has no working-copy commit",
                    self.workspace_name.as_symbol()
                ),
            })?
            .clone();
        let wc_commit =
            repo.store()
                .get_commit(&wc_commit_id)
                .map_err(|e| CoreError::Internal {
                    message: format!("load working-copy commit: {e}"),
                })?;

        let mut locked_ws =
            workspace
                .start_working_copy_mutation()
                .map_err(|e| CoreError::Internal {
                    message: format!("lock working copy: {e}"),
                })?;

        let snapshot_options = SnapshotOptions {
            base_ignores: base_git_ignores(&repo, &self.path)?,
            progress: None,
            start_tracking_matcher: &EverythingMatcher,
            force_tracking_matcher: &NothingMatcher,
            max_new_file_size: u64::MAX,
        };

        let snapshot = locked_ws.locked_wc().snapshot(&snapshot_options);
        let (new_tree, _) = block_on_result("snapshot working copy", snapshot)?;

        if new_tree.tree_ids_and_labels() != wc_commit.tree().tree_ids_and_labels() {
            let mut tx = repo.start_transaction();
            tx.set_is_snapshot(true);
            self.rewrite_commit_tree(
                tx.repo_mut(),
                &wc_commit,
                new_tree,
                "rewrite working-copy commit",
            )?;
            let rebase = tx.repo_mut().rebase_descendants();
            block_on_result("rebase descendants after snapshot", rebase)?;
            let commit = tx.commit("snapshot working copy");
            let new_repo = block_on_result("commit snapshot operation", commit)?;
            block_on_result(
                "finish working-copy snapshot",
                locked_ws.finish(new_repo.op_id().clone()),
            )?;
            self.set_repo(new_repo);
        } else {
            block_on_result(
                "finish clean working-copy snapshot",
                locked_ws.finish(repo.op_id().clone()),
            )?;
            self.set_repo(repo);
        }
        Ok(())
    }

    pub fn has_unignored_working_copy_paths(&self, paths: &[String]) -> CoreResult<bool> {
        let repo = self.get_repo();
        WorkingCopyIgnoreMatcher::new(&repo, self.workspace_name.as_ref(), &self.path)?
            .has_unignored_paths(paths)
    }
}
