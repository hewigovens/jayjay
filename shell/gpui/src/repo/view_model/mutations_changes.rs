use gpui::Context;
use jayjay_core::CoreResult;

use super::RepoViewModel;

impl RepoViewModel {
    pub(crate) fn edit_change(
        &mut self,
        rev: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        let selection = rev.clone();
        self.repo_write_task(
            cx,
            move |repo| repo.edit(&rev),
            move |vm, cx| vm.refresh_selecting_revision(Some(&selection), cx),
        )
    }

    pub(crate) fn squash_change(
        &mut self,
        rev: String,
        into: Option<String>,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        let selection = into.clone();
        self.repo_write_task(
            cx,
            move |repo| repo.squash(&rev, into.as_deref()),
            move |vm, cx| vm.refresh_selecting_revision(selection.as_deref(), cx),
        )
    }

    pub(crate) fn rebase_change(
        &mut self,
        rev: String,
        dest: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        let change_id = self
            .graph
            .changes
            .iter()
            .find(|change| change.change_id.id == rev || change.commit_id.id == rev)
            .map(|change| change.change_id.id.clone())
            .unwrap_or_default();
        let task = self.repo_result_task(
            cx,
            move |repo| repo.rebase(&rev, &dest),
            move |vm, rebased: &String, cx| {
                vm.refresh_preferring(false, Some((change_id, rebased.clone())), cx)
            },
        );
        cx.spawn(async move |_, _| task.await.map(drop))
    }

    pub(crate) fn merge_changes(
        &mut self,
        parents: Vec<String>,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.merge(&parents),
            |vm, cx| vm.refresh_selecting_revision(None, cx),
        )
    }

    pub(crate) fn duplicate_change(
        &mut self,
        rev: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.duplicate(&rev),
            |vm, cx| vm.refresh_selecting_revision(None, cx),
        )
    }

    pub(crate) fn absorb_change(
        &mut self,
        rev: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.absorb(&rev),
            |vm, cx| vm.refresh_selecting_revision(None, cx),
        )
    }

    pub(crate) fn revert_change(
        &mut self,
        rev: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.revert_change(&rev),
            |vm, cx| vm.refresh_selecting_revision(None, cx),
        )
    }
}
