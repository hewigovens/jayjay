use gpui::{AppContext, Context};

use super::RepoViewModel;

impl RepoViewModel {
    pub fn describe_change(&mut self, rev: String, message: String, cx: &mut Context<Self>) {
        let Some(repo) = self.repo.clone() else {
            return;
        };
        self.loading.refreshing = true;
        self.clear_error();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { repo.describe(&rev, &message) })
                .await;
            let _ = this.update(cx, move |vm, cx| {
                vm.loading.refreshing = false;
                match result {
                    Ok(()) => vm.refresh(false, cx),
                    Err(error) => {
                        vm.present_error(error);
                        cx.notify();
                    }
                }
            });
        })
        .detach();
    }

    pub fn commit_working_copy(&mut self, message: String, cx: &mut Context<Self>) {
        let Some(repo) = self.repo.clone() else {
            return;
        };
        self.loading.refreshing = true;
        self.clear_error();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { repo.jj_commit(&message) })
                .await;
            let _ = this.update(cx, move |vm, cx| {
                vm.loading.refreshing = false;
                match result {
                    Ok(()) => {
                        vm.selected = None;
                        vm.refresh(false, cx);
                    }
                    Err(error) => {
                        vm.present_error(error);
                        cx.notify();
                    }
                }
            });
        })
        .detach();
    }
}
