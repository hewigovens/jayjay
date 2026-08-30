use std::sync::Arc;

use jj_lib::commit::Commit;
use jj_lib::merged_tree::MergedTree;
use jj_lib::op_store::RefTarget;
use jj_lib::ref_name::RefName;
use jj_lib::repo::{MutableRepo, ReadonlyRepo};

use super::Repo;
use super::support::block_on_result;
use crate::types::*;

impl Repo {
    pub(crate) fn load_parent_tree(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &Commit,
        context: &str,
    ) -> CoreResult<MergedTree> {
        block_on_result(context, commit.parent_tree(repo.as_ref()))
    }

    pub(crate) fn with_repo_transaction<F>(
        &self,
        description: &str,
        rebase_descendants: bool,
        update: F,
    ) -> CoreResult<()>
    where
        F: FnOnce(&Arc<ReadonlyRepo>, &mut MutableRepo) -> CoreResult<()>,
    {
        let repo = self.get_repo();
        let mut tx = repo.start_transaction();
        update(&repo, tx.repo_mut())?;
        if rebase_descendants {
            self.commit_transaction_rebase(tx, description)
        } else {
            self.commit_transaction(tx, description)
        }
    }

    pub(crate) fn with_existing_commit_transaction<F>(
        &self,
        repo: Arc<ReadonlyRepo>,
        commit: Commit,
        description: &str,
        rebase_descendants: bool,
        update: F,
    ) -> CoreResult<()>
    where
        F: FnOnce(&Arc<ReadonlyRepo>, &Commit, &mut MutableRepo) -> CoreResult<()>,
    {
        let mut tx = repo.start_transaction();
        update(&repo, &commit, tx.repo_mut())?;
        if rebase_descendants {
            self.commit_transaction_rebase(tx, description)
        } else {
            self.commit_transaction(tx, description)
        }
    }

    pub(crate) fn with_resolved_commit_transaction<F>(
        &self,
        rev: &str,
        description: &str,
        rebase_descendants: bool,
        update: F,
    ) -> CoreResult<()>
    where
        F: FnOnce(&Arc<ReadonlyRepo>, &Commit, &mut MutableRepo) -> CoreResult<()>,
    {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        let commit = self.follow_rewrites(&repo, commit, rev)?;
        self.with_existing_commit_transaction(repo, commit, description, rebase_descendants, update)
    }

    pub(crate) fn rewrite_commit_description(
        &self,
        repo_mut: &mut MutableRepo,
        commit: &Commit,
        message: &str,
        context: &str,
    ) -> CoreResult<()> {
        let write = repo_mut
            .rewrite_commit(commit)
            .set_description(message)
            .write();
        block_on_result(context, write)?;
        Ok(())
    }

    pub(crate) fn rewrite_commit_tree(
        &self,
        repo_mut: &mut MutableRepo,
        commit: &Commit,
        tree: MergedTree,
        context: &str,
    ) -> CoreResult<()> {
        let write = repo_mut.rewrite_commit(commit).set_tree(tree).write();
        block_on_result(context, write)?;
        Ok(())
    }

    pub(crate) fn rewrite_existing_commit_with_tree<F>(
        &self,
        repo: Arc<ReadonlyRepo>,
        commit: Commit,
        description: &str,
        rebase_descendants: bool,
        rewrite_context: &str,
        build_tree: F,
    ) -> CoreResult<()>
    where
        F: FnOnce(&Arc<ReadonlyRepo>, &Commit) -> CoreResult<MergedTree>,
    {
        let tree = build_tree(&repo, &commit)?;
        self.with_existing_commit_transaction(
            repo,
            commit,
            description,
            rebase_descendants,
            move |_, commit, repo_mut| {
                self.rewrite_commit_tree(repo_mut, commit, tree, rewrite_context)
            },
        )
    }

    pub(crate) fn edit_working_copy_commit(
        &self,
        repo_mut: &mut MutableRepo,
        commit: &Commit,
        context: &str,
    ) -> CoreResult<()> {
        let edit = repo_mut.edit(self.workspace_name.clone(), commit);
        block_on_result(context, edit)?;
        Ok(())
    }

    pub(crate) fn set_bookmark_target(
        &self,
        repo_mut: &mut MutableRepo,
        name: &str,
        target: RefTarget,
    ) {
        repo_mut.set_local_bookmark_target(RefName::new(name), target);
    }

    pub(crate) fn update_local_bookmark(
        &self,
        name: &str,
        target: RefTarget,
        description: &str,
    ) -> CoreResult<()> {
        self.with_repo_transaction(description, false, move |_, repo_mut| {
            self.set_bookmark_target(repo_mut, name, target);
            Ok(())
        })
    }
}
