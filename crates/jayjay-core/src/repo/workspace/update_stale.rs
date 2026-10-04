use std::sync::Arc;

use jj_lib::op_store::OpStoreError;
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::working_copy::create_and_check_out_recovery_commit;

use super::super::Repo;
use super::super::support::{block_on, block_on_result, load_workspace_internal};
use crate::types::*;

impl Repo {
    /// `jj workspace update-stale`: edits made while stale are recorded at the operation the working copy last saw, so the checkout cannot overwrite them.
    pub fn update_stale_workspace(&self) -> CoreResult<()> {
        let _write = self.write_guard()?;
        match self.refresh_working_copy() {
            Err(CoreError::WorkingCopyStale) => {}
            result => return result,
        }
        let context = "update stale workspace";
        let mut workspace = load_workspace_internal(&self.path, context)?;
        let repo_loader = workspace.repo_loader().clone();
        let mut locked_ws = block_on_result(context, workspace.start_working_copy_mutation())?;
        let last_seen = locked_ws.locked_wc().old_operation_id().clone();
        let repo = match block_on(repo_loader.load_operation(&last_seen)) {
            Ok(operation) => {
                let seen = block_on_result(context, repo_loader.load_at(&operation))?;
                self.record_working_copy(&mut locked_ws, seen)?;
                let head = block_on_result(context, repo_loader.load_at_head())?;
                let wc_commit = self.working_copy_commit(&head)?;
                block_on_result(context, locked_ws.locked_wc().check_out(&wc_commit))?;
                head
            }
            Err(OpStoreError::ObjectNotFound { .. }) => {
                let head = block_on_result(context, repo_loader.load_at_head())?;
                let recovery = create_and_check_out_recovery_commit(
                    locked_ws.locked_wc(),
                    &head,
                    self.workspace_name.clone(),
                    "RECOVERY COMMIT FROM `jj workspace update-stale`",
                );
                block_on_result(context, recovery)?.0
            }
            Err(error) => return Err(CoreError::internal(format!("{context}: {error}"))),
        };
        let repo = self.reset_colocated_git_head(repo)?;
        block_on_result(context, locked_ws.finish(repo.op_id().clone()))?;
        self.set_repo(repo);
        Ok(())
    }

    /// The checkout moved `@` under a new parent without a transaction, so Git HEAD still names the old one; the CLI records this same operation.
    fn reset_colocated_git_head(&self, repo: Arc<ReadonlyRepo>) -> CoreResult<Arc<ReadonlyRepo>> {
        if !self.is_colocated(repo.store()) {
            return Ok(repo);
        }
        let mut tx = repo.start_transaction();
        self.sync_colocated_git(&mut tx)?;
        block_on_result("reset git head", tx.commit("reset git head"))
    }
}
