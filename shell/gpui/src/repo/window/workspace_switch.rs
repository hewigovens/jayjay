use std::path::{Path, PathBuf};

use gpui::{Context, Entity};
use jayjay_core::repositories::normalize_repository_path;

use super::RepoWindow;
use super::open::{activate_repo_window, retitle_and_focus};
use super::workspace_drafts::WorkspaceDrafts;
use crate::repo::view_model::RepoViewModel;

impl RepoWindow {
    pub fn switch_workspace(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let target = normalize_repository_path(&path);
        self.switching = None;
        if target == self.normalized_repo_path(cx) {
            return;
        }
        self.switching = Some(cx.spawn(async move |this, cx| {
            if cx.update(|cx| activate_repo_window(&target, cx)) {
                return;
            }
            let opened = RepoViewModel::open_detached(target.clone(), cx).await;
            // No await between this recheck and the swap, so no window can open the target in between.
            if cx.update(|cx| activate_repo_window(&target, cx)) {
                return;
            }
            let _ = this.update(cx, |view, cx| match opened {
                Ok(vm) => view.replace_workspace(target, vm, cx),
                Err(error) => view.vm.update(cx, |vm, cx| {
                    vm.present_error(error);
                    cx.notify();
                }),
            });
        }));
    }

    fn replace_workspace(
        &mut self,
        target: PathBuf,
        vm: Entity<RepoViewModel>,
        cx: &mut Context<Self>,
    ) {
        if self.vm.read(cx).loading.operations > 0 {
            self.show_toast(
                "Finish the running operation before switching workspaces",
                cx,
            );
            return;
        }
        let window = cx.entity_id();
        let draft = self.unsaved_commit_draft(cx);
        WorkspaceDrafts::preserve(window, self.normalized_repo_path(cx), draft, cx);
        let restored = WorkspaceDrafts::take(window, &target, cx);

        self.cancel_pending_commit_message_generation();
        let mut fresh = Self::for_vm(vm, cx);
        fresh.boot(cx);
        fresh.focus_handle = self.focus_handle.clone();
        fresh.commit_ai = std::mem::take(&mut self.commit_ai);
        if let Some(draft) = restored {
            fresh.restore_commit_draft(draft, cx);
        }
        *self = fresh;
        self.vm.update(cx, |vm, cx| vm.finish_open(cx));

        let view = cx.entity();
        cx.defer(move |cx| retitle_and_focus(&view, &target, cx));
        cx.notify();
    }

    fn normalized_repo_path(&self, cx: &gpui::App) -> PathBuf {
        normalize_repository_path(Path::new(self.vm.read(cx).repo_path.as_ref()))
    }
}
