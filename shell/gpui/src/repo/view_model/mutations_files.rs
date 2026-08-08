//! Batch file mutations dispatched from the file-column context menu: restore to parent, delete from disk, ignore & untrack.

use gpui::Context;
use jayjay_core::CoreResult;

use super::RepoViewModel;

impl RepoViewModel {
    /// Discard `paths` in `rev` back to a parent's content: `from` picks which parent on a merge (`jj restore --from`), `None` uses the auto-merged parent tree.
    pub(crate) fn restore_files(
        &mut self,
        rev: String,
        from: Option<String>,
        paths: Vec<String>,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.restore_files(&rev, from.as_deref(), &paths),
            |vm, cx| vm.refresh(false, cx),
        )
    }

    /// Delete working-copy files from disk; jj picks the deletions up on the snapshot the refresh triggers.
    pub(crate) fn delete_files(
        &mut self,
        paths: Vec<String>,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.delete_files(&paths),
            |vm, cx| vm.refresh(false, cx),
        )
    }

    /// Append `paths` to `.gitignore` and `jj file untrack` them.
    pub(crate) fn ignore_and_untrack(
        &mut self,
        paths: Vec<String>,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.ignore_and_untrack(&paths),
            |vm, cx| vm.refresh(false, cx),
        )
    }
}
