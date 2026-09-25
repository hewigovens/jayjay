use jj_lib::merged_tree::MergedTree;
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::store::Store;
use jj_lib::transaction::Transaction;

use crate::repo::Repo;
use crate::repo::support::block_on_result;
use crate::types::*;

impl Repo {
    pub(crate) fn is_colocated(&self, store: &Store) -> bool {
        jj_lib::git::get_git_backend(store)
            .is_ok_and(|backend| backend.open_git_repo_at_workdir(&self.path).is_ok())
    }

    /// A colocated checkout expects Git HEAD at @'s parent and Git refs at the bookmarks; the CLI does this after every command, jj-lib leaves it to the caller.
    pub(crate) fn sync_colocated_git(&self, tx: &mut Transaction) -> CoreResult<()> {
        let repo_mut = tx.repo_mut();
        if !self.is_colocated(repo_mut.store()) {
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

    /// Files a snapshot started tracking get intent-to-add entries so `git status` and `git diff` see them, as they would after a CLI snapshot.
    pub(crate) fn sync_colocated_index(
        &self,
        repo: &ReadonlyRepo,
        old_tree: &MergedTree,
        new_tree: &MergedTree,
    ) -> CoreResult<()> {
        if !self.is_colocated(repo.store()) {
            return Ok(());
        }
        block_on_result(
            "update git index",
            jj_lib::git::update_intent_to_add(repo, &self.path, old_tree, new_tree),
        )
    }
}
