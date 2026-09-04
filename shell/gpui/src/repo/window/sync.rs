use std::future::Future;

use gpui::{AppContext, AsyncApp, Context};
use jayjay_core::FetchResult;
use jayjay_core::repositories::normalize_repository_path;

use std::path::Path;

use super::RepoWindow;
use super::confirmation::{Confirmation, ConfirmedAction};

impl RepoWindow {
    pub fn git_fetch_origin(&mut self, cx: &mut Context<Self>) {
        if self.sync_activity.fetching {
            self.show_toast("Pull already in progress", cx);
            return;
        }
        self.sync_activity.fetching = true;
        cx.notify();
        let task = self.vm.update(cx, |vm, cx| vm.git_fetch_origin(cx));
        Self::spawn_update(
            cx,
            |_| task,
            |view, result, cx| {
                view.sync_activity.fetching = false;
                let outcome = match result {
                    Ok(result) => {
                        let message = fetch_status_message(&result);
                        view.show_toast(message.clone(), cx);
                        Ok(message)
                    }
                    Err(error) => Err(error.to_string()),
                };
                view.notify_sync_completion(SyncOperation::Fetch, outcome, cx);
                cx.notify();
            },
        );
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
                "No stale bookmarks found".to_owned()
            } else {
                format!(
                    "Forgot {count} stale bookmark{}",
                    if count == 1 { "" } else { "s" }
                )
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
                cx.defer(move |cx| super::open::close_repo_window_at(Path::new(&path), cx));
            }
            view.show_toast(format!("Forgot workspace {name}"), cx);
        });
    }

    pub(super) fn request_workspace_delete(
        &mut self,
        name: String,
        path: String,
        cx: &mut Context<Self>,
    ) {
        self.request_confirmation(
            Confirmation {
                title: format!("Delete Workspace {name}?").into(),
                message: format!(
                    "This closes its window, forgets the workspace, and deletes its directory from disk:\n{path}"
                )
                .into(),
                confirm_label: "Delete".into(),
                action: ConfirmedAction::DeleteWorkspace { name, path },
            },
            cx,
        );
    }

    pub(super) fn delete_workspace(&mut self, name: String, path: String, cx: &mut Context<Self>) {
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
            match warning {
                Some(warning) => view.vm.update(cx, |vm, _| vm.present_error(warning)),
                None => view.show_toast(format!("Deleted workspace {name}"), cx),
            }
        });
    }

    pub(crate) fn open_repo_in_editor(&mut self, cx: &mut Context<Self>) {
        let repo_path = self.vm.read(cx).repo_path.to_string();
        if !crate::app::tools::open_in_editor(&repo_path, ".", cx) {
            self.show_toast("Editor could not be opened", cx);
        }
    }

    pub(crate) fn open_repo_in_terminal(&mut self, cx: &mut Context<Self>) {
        let repo_path = self.vm.read(cx).repo_path.to_string();
        if !crate::app::tools::open_in_terminal(&repo_path, cx) {
            self.show_toast("Terminal could not be opened", cx);
        }
    }

    pub(crate) fn show_repo_in_file_manager(&mut self, cx: &mut Context<Self>) {
        let repo_path = self.vm.read(cx).repo_path.to_string();
        if !crate::app::tools::show_in_file_manager(&repo_path, None) {
            self.show_toast("Repository could not be shown in the file manager", cx);
        }
    }

    pub(crate) fn open_remote_repository(&mut self, cx: &mut Context<Self>) {
        let Some(repo) = self.vm.read(cx).repo.clone() else {
            self.show_toast("Repository is not open", cx);
            return;
        };
        Self::spawn_update(
            cx,
            move |cx| cx.background_spawn(async move { repo.remote_web_url() }),
            |view, url, cx| {
                if let Some(url) = url {
                    crate::app::links::open_url(cx, &url);
                } else {
                    view.show_toast("Couldn't determine a web URL for origin", cx);
                }
            },
        );
    }

    pub(crate) fn git_push_bookmark(&mut self, bookmark: String, cx: &mut Context<Self>) -> bool {
        if self.sync_activity.pushing {
            self.show_toast("Push already in progress", cx);
            return false;
        }
        self.sync_activity.pushing = true;
        cx.notify();
        let task = self
            .vm
            .update(cx, |vm, cx| vm.push_bookmark(bookmark.clone(), cx));
        Self::spawn_update(
            cx,
            |_| task,
            move |view, result, cx| {
                view.sync_activity.pushing = false;
                let outcome = match result {
                    Ok(message) => {
                        let message = push_status_message(&bookmark, &message);
                        view.show_toast(message.clone(), cx);
                        Ok(message)
                    }
                    Err(error) => Err(error.to_string()),
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
            crate::platform::send_notification(title, &body);
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
) -> Option<(&'static str, String)> {
    if window_active {
        return None;
    }
    Some(match (operation, outcome) {
        (SyncOperation::Fetch, Ok(message)) => ("Fetch complete", message),
        (SyncOperation::Fetch, Err(message)) => ("Fetch failed", message),
        (SyncOperation::Push, Ok(message)) => ("Push complete", message),
        (SyncOperation::Push, Err(message)) => ("Push failed", message),
    })
}

fn fetch_status_message(result: &FetchResult) -> String {
    let mut message = result.message.trim().to_owned();
    if message.is_empty() {
        message = "Fetched origin".to_owned();
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
                "Pushed bookmarks".to_owned()
            } else {
                format!("Pushed {bookmark}")
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
            Some(("Push failed", "authentication failed".to_owned()))
        );
    }
}
