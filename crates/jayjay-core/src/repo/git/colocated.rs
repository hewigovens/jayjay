use jj_lib::repo::Repo as _;
use jj_lib::transaction::Transaction;

use crate::repo::Repo;
use crate::repo::support::block_on_result;
use crate::types::*;

impl Repo {
    /// A colocated checkout expects Git HEAD at @'s parent and Git refs at the bookmarks; the CLI does this after every command, jj-lib leaves it to the caller.
    pub(crate) fn sync_colocated_git(&self, tx: &mut Transaction) -> CoreResult<()> {
        let repo_mut = tx.repo_mut();
        let colocated = jj_lib::git::get_git_backend(repo_mut.store())
            .is_ok_and(|backend| backend.open_git_repo_at_workdir(&self.path).is_ok());
        if !colocated {
            return Ok(());
        }
        if let Some(commit_id) = repo_mut
            .view()
            .get_wc_commit_id(self.workspace_name.as_ref())
            .cloned()
        {
            let wc_commit = repo_mut.store().get_commit(&commit_id).map_err(|error| {
                CoreError::internal(format!("load working-copy commit: {error}"))
            })?;
            block_on_result(
                "reset git head",
                jj_lib::git::reset_head(repo_mut, &self.workspace_name, &self.path, &wc_commit),
            )?;
        }
        jj_lib::git::export_refs(repo_mut)
            .map_err(|error| CoreError::internal(format!("export git refs: {error}")))?;
        Ok(())
    }
}
