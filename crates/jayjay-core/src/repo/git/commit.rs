use crate::repo::Repo;
use crate::repo::support::block_on_result;
use crate::types::*;

impl Repo {
    /// `jj commit -m <message>`: describe `@`, then start a new empty change on top of it.
    pub fn jj_commit(&self, message: &str) -> CoreResult<()> {
        let _write = self.write_guard()?;
        self.refresh_working_copy()?;
        self.with_resolved_commit_transaction("@", "commit", true, |repo, commit, repo_mut| {
            self.ensure_commit_mutable(repo, commit, "@")?;
            let described = repo_mut
                .rewrite_commit(commit)
                .set_description(message)
                .write();
            let described = block_on_result("commit", described)?;
            let new_commit = repo_mut
                .new_commit(vec![described.id().clone()], described.tree())
                .write();
            let new_commit = block_on_result("commit", new_commit)?;
            self.edit_working_copy_commit(repo_mut, &new_commit, "edit working copy")
        })
    }
}
