use std::future::Future;

use gpui::{AppContext, AsyncApp, Context};
use jayjay_core::FetchResult;

use super::RepoWindow;

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
                if let Ok(result) = result {
                    view.show_toast(fetch_status_message(&result), cx);
                }
                cx.notify();
            },
        );
    }

    pub fn git_push_default(&mut self, cx: &mut Context<Self>) {
        self.git_push_bookmark(String::new(), cx);
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

    pub(crate) fn forget_workspace(&mut self, name: String, cx: &mut Context<Self>) {
        let task = self
            .vm
            .update(cx, |vm, cx| vm.workspace_forget(name.clone(), cx));
        Self::spawn_ok(cx, task, move |view, _, cx| {
            view.show_toast(format!("Forgot workspace {name}"), cx);
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
                    cx.open_url(&url);
                } else {
                    view.show_toast("Couldn't determine a web URL for origin", cx);
                }
            },
        );
    }

    pub(super) fn git_push_bookmark(&mut self, bookmark: String, cx: &mut Context<Self>) {
        if self.sync_activity.pushing {
            self.show_toast("Push already in progress", cx);
            return;
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
                if let Ok(message) = result {
                    view.show_toast(push_status_message(&bookmark, &message), cx);
                }
                cx.notify();
            },
        );
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
