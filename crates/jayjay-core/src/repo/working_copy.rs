use jj_lib::gitignore::GitIgnoreFile;
use jj_lib::matchers::{EverythingMatcher, NothingMatcher};
use jj_lib::repo::Repo as _;
use jj_lib::working_copy::SnapshotOptions;
use jj_lib::workspace::Workspace;
use pollster::FutureExt as _;

use super::Repo;
use super::config::{default_settings, working_copy_factories};
use crate::types::*;
use jj_lib::repo::StoreFactories;

impl Repo {
    pub fn refresh_working_copy(&self) -> CoreResult<()> {
        let settings = default_settings()?;
        let store_factories = StoreFactories::default();
        let wc_factories = working_copy_factories();
        let mut workspace = Workspace::load(&settings, &self.path, &store_factories, &wc_factories)
            .map_err(|e| CoreError::Internal {
                message: format!("load workspace for snapshot: {e}"),
            })?;

        let repo = workspace
            .repo_loader()
            .load_at_head()
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("load repo for snapshot: {e}"),
            })?;

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
            base_ignores: GitIgnoreFile::empty(),
            progress: None,
            start_tracking_matcher: &EverythingMatcher,
            force_tracking_matcher: &NothingMatcher,
            max_new_file_size: u64::MAX,
        };

        let (new_tree, _) = locked_ws
            .locked_wc()
            .snapshot(&snapshot_options)
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("snapshot working copy: {e}"),
            })?;

        if new_tree.tree_ids_and_labels() != wc_commit.tree().tree_ids_and_labels() {
            let mut tx = repo.start_transaction();
            tx.set_is_snapshot(true);
            tx.repo_mut()
                .rewrite_commit(&wc_commit)
                .set_tree(new_tree)
                .write()
                .block_on()
                .map_err(|e| CoreError::Internal {
                    message: format!("rewrite working-copy commit: {e}"),
                })?;
            tx.repo_mut()
                .rebase_descendants()
                .block_on()
                .map_err(|e| CoreError::Internal {
                    message: format!("rebase descendants after snapshot: {e}"),
                })?;
            let new_repo =
                tx.commit("snapshot working copy")
                    .block_on()
                    .map_err(|e| CoreError::Internal {
                        message: format!("commit snapshot operation: {e}"),
                    })?;
            locked_ws
                .finish(new_repo.op_id().clone())
                .map_err(|e| CoreError::Internal {
                    message: format!("finish working-copy snapshot: {e}"),
                })?;
            self.set_repo(new_repo);
        } else {
            locked_ws
                .finish(repo.op_id().clone())
                .map_err(|e| CoreError::Internal {
                    message: format!("finish clean working-copy snapshot: {e}"),
                })?;
            self.set_repo(repo);
        }
        Ok(())
    }
}
