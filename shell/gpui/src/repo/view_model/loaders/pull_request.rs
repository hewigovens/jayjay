use gpui::Context;
use jayjay_core::ChangeInfo;

use super::super::RepoViewModel;

impl RepoViewModel {
    pub(in crate::repo) fn refresh_pr_info(&mut self, change: &ChangeInfo, cx: &mut Context<Self>) {
        let Some(repo) = self.repo.clone() else {
            return;
        };
        let Some(bookmark) = change.bookmarks.first().cloned() else {
            return;
        };
        self.loading.pr_gen = self.loading.pr_gen.wrapping_add(1);
        let generation = self.loading.pr_gen;
        self.loading.pr = true;
        Self::background_update(
            cx,
            async move { repo.pull_request_info(&bookmark) },
            move |vm, info, cx| {
                // A newer selection's fetch superseded this one; its result lands later.
                if vm.loading.pr_gen != generation {
                    return;
                }
                vm.loading.pr = false;
                vm.pr_info = info;
                cx.notify();
            },
        );
    }
}
