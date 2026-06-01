use gpui::Context;
use jayjay_core::{CoreResult, init_jj_git_repo};

use super::RepoViewModel;

impl RepoViewModel {
    pub fn describe_change(
        &mut self,
        rev: String,
        message: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.describe(&rev, &message),
            |vm, cx| vm.refresh(false, cx),
        )
    }

    pub fn commit_working_copy(
        &mut self,
        message: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.jj_commit(&message),
            |vm, cx| {
                vm.selected = None;
                vm.refresh(false, cx);
            },
        )
    }

    pub fn new_change_on_top(
        &mut self,
        parent: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.new_change(&parent, ""),
            |vm, cx| {
                vm.selected = None;
                vm.refresh(false, cx);
            },
        )
    }

    pub fn abandon_change(
        &mut self,
        rev: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.abandon(&rev),
            |vm, cx| {
                vm.selected = None;
                vm.refresh(false, cx);
            },
        )
    }

    pub fn move_bookmark_to_parent(
        &mut self,
        name: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<()>> {
        self.repo_write_task(
            cx,
            move |repo| repo.move_bookmark(&name, "@-"),
            |vm, cx| vm.refresh(false, cx),
        )
    }

    pub fn push_bookmark(
        &mut self,
        name: String,
        cx: &mut Context<Self>,
    ) -> gpui::Task<CoreResult<String>> {
        self.repo_result_task(
            cx,
            move |repo| repo.git_push(&name),
            |vm, _message, cx| vm.refresh(false, cx),
        )
    }

    pub fn initialize_repo(&mut self, cx: &mut Context<Self>) -> gpui::Task<CoreResult<()>> {
        let path = std::path::PathBuf::from(self.repo_path.as_ref());
        self.loading.refreshing = true;
        self.clear_error();
        cx.notify();

        Self::core_result_task(
            cx,
            {
                let path = path.clone();
                async move { init_jj_git_repo(&path) }
            },
            move |vm, result, cx| match result {
                Ok(()) => {
                    *vm = RepoViewModel::new(path);
                    vm.boot(cx);
                    cx.notify();
                    Ok(())
                }
                Err(error) => {
                    vm.loading.refreshing = false;
                    vm.present_error(&error);
                    cx.notify();
                    Err(error)
                }
            },
        )
    }
}
