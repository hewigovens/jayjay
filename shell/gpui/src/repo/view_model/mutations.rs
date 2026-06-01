use gpui::{AppContext, Context};
use jayjay_core::{CoreResult, Error, init_jj_git_repo};

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

    pub fn commit_working_copy(
        &mut self,
        message: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        let Some(repo) = self.repo.clone() else {
            self.present_error("repository is not open");
            cx.notify();
            return cx.spawn(async move |_, _| Err(Error::internal("repository is not open")));
        };
        self.loading.refreshing = true;
        self.clear_error();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { repo.jj_commit(&message) })
                .await;
            let _ = this.update(cx, |vm, cx| {
                vm.loading.refreshing = false;
                match &result {
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
            result
        })
    }

    pub fn initialize_repo(&mut self, cx: &mut Context<Self>) -> gpui::Task<CoreResult<()>> {
        let path = std::path::PathBuf::from(self.repo_path.as_ref());
        self.loading.refreshing = true;
        self.clear_error();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn({
                    let path = path.clone();
                    async move { init_jj_git_repo(&path) }
                })
                .await;
            match result {
                Ok(()) => {
                    let _ = this.update(cx, move |vm, cx| {
                        *vm = RepoViewModel::new(path);
                        vm.boot(cx);
                        cx.notify();
                    });
                    Ok(())
                }
                Err(error) => {
                    let message = format!("{error}");
                    let _ = this.update(cx, move |vm, cx| {
                        vm.loading.refreshing = false;
                        vm.present_error(message);
                        cx.notify();
                    });
                    Err(error)
                }
            }
        })
    }
}
