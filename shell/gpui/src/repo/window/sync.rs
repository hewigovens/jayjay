use std::future::Future;

use gpui::{AppContext, AsyncApp, Context, Task};
use jayjay_core::repositories::normalize_repository_path;
use jayjay_core::{FetchResult, JayError, JayResult, SyncToken};

use std::path::Path;

use super::RepoWindow;
use super::confirmation::{Confirmation, ConfirmedAction, DontAskAgain};
use super::workspace_drafts::WorkspaceDrafts;
use crate::app::config;
use crate::repo::view_model::RepoViewModel;

type PushTask = fn(
    &mut RepoViewModel,
    String,
    SyncToken,
    &mut Context<RepoViewModel>,
) -> Task<JayResult<String>>;

impl RepoWindow {
    pub fn git_fetch_origin(&mut self, cx: &mut Context<Self>) {
        self.start_fetch(cx, |vm, sync, cx| vm.git_fetch_origin(sync, cx));
    }

    pub fn git_pull_bookmark(&mut self, bookmark: String, cx: &mut Context<Self>) {
        self.start_fetch(cx, move |vm, sync, cx| {
            vm.git_pull_bookmark(bookmark, sync, cx)
        });
    }

    fn start_fetch(
        &mut self,
        cx: &mut Context<Self>,
        fetch: impl FnOnce(
            &mut RepoViewModel,
            SyncToken,
            &mut Context<RepoViewModel>,
        ) -> Task<JayResult<FetchResult>>,
    ) {
        if self.sync_activity.is_fetching() {
            self.show_toast(jayjay_i18n::repo::pull_already_in_progress(), cx);
            return;
        }
        let Some(sync) = self.new_sync_token(cx) else {
            return;
        };
        self.sync_activity.fetch = Some(sync.clone());
        cx.notify();
        let task = self.vm.update(cx, |vm, cx| fetch(vm, sync, cx));
        Self::spawn_update(
            cx,
            |_| task,
            |view, result, cx| {
                view.sync_activity.fetch = None;
                let outcome = match result {
                    Ok(result) => {
                        let message = fetch_status_message(&result);
                        view.show_toast(message.clone(), cx);
                        Ok(message)
                    }
                    Err(JayError::Canceled) => {
                        view.finish_canceled_sync(jayjay_i18n::repo::pull_canceled(), cx);
                        return;
                    }
                    Err(error) => Err(crate::app::error_text(error).to_string()),
                };
                view.notify_sync_completion(SyncOperation::Fetch, outcome, cx);
                cx.notify();
            },
        );
    }

    pub fn pull_or_cancel(&mut self, cx: &mut Context<Self>) {
        if self.sync_activity.is_fetching() {
            self.cancel_pull(cx);
        } else {
            self.git_fetch_origin(cx);
        }
    }

    pub fn push_or_cancel(&mut self, cx: &mut Context<Self>) {
        if self.sync_activity.is_pushing() {
            self.cancel_push(cx);
        } else {
            self.git_push_default(cx);
        }
    }

    pub(crate) fn cancel_pull(&mut self, _cx: &mut Context<Self>) {
        if let Some(sync) = &self.sync_activity.fetch {
            sync.cancel();
        }
    }

    pub(crate) fn cancel_push(&mut self, _cx: &mut Context<Self>) {
        if let Some(sync) = &self.sync_activity.push {
            sync.cancel();
        }
    }

    fn new_sync_token(&mut self, cx: &mut Context<Self>) -> Option<SyncToken> {
        let sync = self.vm.read(cx).repo.as_ref().map(|repo| repo.sync_token());
        if sync.is_none() {
            self.show_toast(jayjay_i18n::repo::repository_not_open(), cx);
        }
        sync
    }

    fn finish_canceled_sync(&mut self, message: String, cx: &mut Context<Self>) {
        self.show_toast(message, cx);
        // The remote side may have landed before the cancel took effect.
        self.vm.update(cx, |vm, cx| vm.refresh(false, cx));
        cx.notify();
    }

    pub fn git_push_default(&mut self, cx: &mut Context<Self>) {
        self.git_push_bookmark(String::new(), cx);
    }

    pub fn confirm_pending_push(&mut self, cx: &mut Context<Self>) {
        let Some(bookmark) = self.feedback.pending_push_bookmark.clone() else {
            return;
        };
        if self.git_push_bookmark(bookmark.to_string(), cx) {
            self.feedback.pending_push_bookmark = None;
            cx.notify();
        }
    }

    pub fn dismiss_pending_push(&mut self, cx: &mut Context<Self>) {
        self.feedback.pending_push_bookmark = None;
        cx.notify();
    }

    pub(crate) fn forget_stale_bookmarks(&mut self, cx: &mut Context<Self>) {
        let task = self.vm.update(cx, |vm, cx| vm.forget_stale_bookmarks(cx));
        Self::spawn_ok(cx, task, |view, count, cx| {
            let message = if count == 0 {
                jayjay_i18n::repo::no_stale_bookmarks_found()
            } else {
                jayjay_i18n::repo::forgot_stale_bookmarks(count)
            };
            view.show_toast(message, cx);
        });
    }

    pub(crate) fn forget_workspace(
        &mut self,
        name: String,
        path: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let expected_root = path.clone();
        let task = self.vm.update(cx, |vm, cx| {
            vm.workspace_forget(name.clone(), expected_root, cx)
        });
        Self::spawn_ok(cx, task, move |view, _, cx| {
            if let Some(path) = path {
                WorkspaceDrafts::discard(Path::new(&path), cx);
                cx.defer(move |cx| super::open::close_repo_window_at(Path::new(&path), cx));
            }
            view.show_toast(jayjay_i18n::repo::forgot_workspace(&name), cx);
        });
    }

    pub(super) fn request_workspace_delete(
        &mut self,
        name: String,
        path: String,
        cx: &mut Context<Self>,
    ) {
        if config::current(cx)
            .features
            .skip_workspace_delete_confirmation
        {
            self.delete_workspace(name, path, cx);
            return;
        }
        self.request_confirmation(
            Confirmation {
                title: jayjay_i18n::repo::delete_workspace_title(&name).into(),
                message: jayjay_i18n::repo::delete_workspace_message(&path).into(),
                confirm_label: jayjay_i18n::repo::delete().into(),
                action: ConfirmedAction::DeleteWorkspace { name, path },
                dont_ask_again: Some(DontAskAgain {
                    is_set: |config| config.features.skip_workspace_delete_confirmation,
                    toggle: |config| config.features.skip_workspace_delete_confirmation ^= true,
                }),
            },
            cx,
        );
    }

    pub(crate) fn delete_workspace(&mut self, name: String, path: String, cx: &mut Context<Self>) {
        let closing = path.clone();
        cx.defer(move |cx| super::open::close_repo_window_at(Path::new(&closing), cx));
        let recent_entry = normalize_repository_path(Path::new(&path))
            .to_string_lossy()
            .into_owned();
        let task = self.vm.update(cx, |vm, cx| {
            vm.workspace_forget_and_delete(name.clone(), path.clone(), cx)
        });
        Self::spawn_ok(cx, task, move |view, warning, cx| {
            crate::app::config::update(cx, |config| config.remove_recent_repo(&recent_entry));
            crate::app::repositories::set_pinned(cx, Path::new(&recent_entry), false);
            WorkspaceDrafts::discard(Path::new(&recent_entry), cx);
            match warning {
                Some(warning) => view.vm.update(cx, |vm, _| vm.present_error(warning)),
                None => view.show_toast(jayjay_i18n::repo::deleted_workspace(&name), cx),
            }
        });
    }

    pub(crate) fn open_repo_in_editor(&mut self, cx: &mut Context<Self>) {
        let repo_path = self.vm.read(cx).repo_path.to_string();
        if !crate::app::tools::open_in_editor(&repo_path, ".", None, cx) {
            self.show_toast(jayjay_i18n::repo::editor_could_not_be_opened(), cx);
        }
    }

    pub(crate) fn open_repo_in_terminal(&mut self, cx: &mut Context<Self>) {
        let repo_path = self.vm.read(cx).repo_path.to_string();
        if !crate::app::tools::open_in_terminal(&repo_path, cx) {
            self.show_toast(jayjay_i18n::repo::terminal_could_not_be_opened(), cx);
        }
    }

    pub(crate) fn show_repo_in_file_manager(&mut self, cx: &mut Context<Self>) {
        let repo_path = self.vm.read(cx).repo_path.to_string();
        if !crate::app::tools::show_in_file_manager(&repo_path, None) {
            self.show_toast(jayjay_i18n::repo::repository_could_not_be_shown_in(), cx);
        }
    }

    pub(crate) fn open_remote_repository(&mut self, cx: &mut Context<Self>) {
        let Some(repo) = self.vm.read(cx).repo.clone() else {
            self.show_toast(jayjay_i18n::repo::repository_not_open(), cx);
            return;
        };
        Self::spawn_update(
            cx,
            move |cx| cx.background_spawn(async move { repo.remote_web_url() }),
            |view, url, cx| {
                if let Some(url) = url {
                    crate::app::links::open_url(cx, &url);
                } else {
                    view.show_toast(jayjay_i18n::repo::couldnt_determine_a_web_url_for(), cx);
                }
            },
        );
    }

    pub(crate) fn git_push_bookmark(&mut self, bookmark: String, cx: &mut Context<Self>) -> bool {
        self.git_push(bookmark, RepoViewModel::push_bookmark, cx)
    }

    /// `name` is the bookmark or tag to push; an empty name pushes every tracked bookmark.
    pub(super) fn git_push(
        &mut self,
        name: String,
        push: PushTask,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.sync_activity.is_pushing() {
            self.show_toast(jayjay_i18n::repo::push_already_in_progress(), cx);
            return false;
        }
        let Some(sync) = self.new_sync_token(cx) else {
            return false;
        };
        self.sync_activity.push = Some(sync.clone());
        cx.notify();
        let task = self
            .vm
            .update(cx, |vm, cx| push(vm, name.clone(), sync, cx));
        Self::spawn_update(
            cx,
            |_| task,
            move |view, result, cx| {
                view.sync_activity.push = None;
                let outcome = match result {
                    Ok(message) => {
                        let message = push_status_message(&name, &message);
                        view.show_toast(message.clone(), cx);
                        Ok(message)
                    }
                    Err(JayError::Canceled) => {
                        view.finish_canceled_sync(jayjay_i18n::repo::push_canceled(), cx);
                        return;
                    }
                    Err(error) => Err(crate::app::error_text(error).to_string()),
                };
                view.notify_sync_completion(SyncOperation::Push, outcome, cx);
                cx.notify();
            },
        );
        true
    }

    fn notify_sync_completion(
        &self,
        operation: SyncOperation,
        outcome: Result<String, String>,
        cx: &mut Context<Self>,
    ) {
        let active = cx
            .active_window()
            .and_then(|window| window.downcast::<Self>())
            .and_then(|window| window.entity(cx).ok())
            .is_some_and(|root| root == cx.entity());
        let Some((title, body)) = sync_completion_notification(active, operation, outcome) else {
            return;
        };
        cx.background_spawn(async move {
            crate::platform::send_notification(&title, &body);
        })
        .detach();
    }

    fn spawn_ok<T, E, Fut>(
        cx: &mut Context<Self>,
        task: Fut,
        update: impl FnOnce(&mut Self, T, &mut Context<Self>) + 'static,
    ) where
        T: 'static,
        E: 'static,
        Fut: Future<Output = Result<T, E>> + 'static,
    {
        Self::spawn_update(
            cx,
            |_| task,
            move |view, result, cx| {
                if let Ok(value) = result {
                    update(view, value, cx);
                }
            },
        );
    }

    fn spawn_update<T, Fut>(
        cx: &mut Context<Self>,
        task: impl FnOnce(&mut AsyncApp) -> Fut + 'static,
        update: impl FnOnce(&mut Self, T, &mut Context<Self>) + 'static,
    ) where
        T: 'static,
        Fut: Future<Output = T> + 'static,
    {
        cx.spawn(async move |this, cx| {
            let value = task(cx).await;
            let _ = this.update(cx, move |view, cx| update(view, value, cx));
        })
        .detach();
    }
}

#[derive(Clone, Copy)]
enum SyncOperation {
    Fetch,
    Push,
}

fn sync_completion_notification(
    window_active: bool,
    operation: SyncOperation,
    outcome: Result<String, String>,
) -> Option<(String, String)> {
    if window_active {
        return None;
    }
    Some(match (operation, outcome) {
        (SyncOperation::Fetch, Ok(message)) => (jayjay_i18n::repo::fetch_complete(), message),
        (SyncOperation::Fetch, Err(message)) => (jayjay_i18n::repo::fetch_failed(), message),
        (SyncOperation::Push, Ok(message)) => (jayjay_i18n::repo::push_complete(), message),
        (SyncOperation::Push, Err(message)) => (jayjay_i18n::repo::push_failed(), message),
    })
}

fn fetch_status_message(result: &FetchResult) -> String {
    let mut message = result.message.trim().to_owned();
    if message.is_empty() {
        message = jayjay_i18n::repo::fetched_origin();
    }
    if !result.abandoned_bookmarks.is_empty() {
        message.push_str("\nAbandoned merged: ");
        message.push_str(&result.abandoned_bookmarks.join(", "));
    }
    if !result.suggest_abandon_bookmarks.is_empty() {
        message.push_str("\nConflicting (may be merged): ");
        message.push_str(&result.suggest_abandon_bookmarks.join(", "));
        message.push_str(" - consider abandoning");
    }
    message
}

fn push_status_message(bookmark: &str, message: &str) -> String {
    message
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| {
            if bookmark.is_empty() {
                jayjay_i18n::repo::pushed_bookmarks()
            } else {
                jayjay_i18n::repo::pushed_bookmark(bookmark)
            }
        })
}

#[cfg(test)]
mod tests {
    use super::{SyncOperation, sync_completion_notification};

    #[test]
    fn sync_completion_notifies_only_for_inactive_windows() {
        assert!(
            sync_completion_notification(true, SyncOperation::Fetch, Ok("Fetched".to_owned()))
                .is_none()
        );
        assert_eq!(
            sync_completion_notification(
                false,
                SyncOperation::Push,
                Err("authentication failed".to_owned()),
            ),
            Some(("Push failed".to_owned(), "authentication failed".to_owned()))
        );
    }
}
