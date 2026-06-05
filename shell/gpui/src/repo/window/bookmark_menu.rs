use gpui::{App, AppContext, Context, SharedString};

use super::RepoWindow;
use crate::repo::revset;
use crate::ui::context_menu::{ContextAction, ContextMenuItem};
use crate::ui::icons::glyph;

impl RepoWindow {
    pub(super) fn build_bookmark_menu(&self, name: &str, cx: &App) -> Vec<ContextMenuItem> {
        let pull_request_label = self.pull_request_menu_label(cx);
        let mut items = vec![
            ContextMenuItem::new(
                "Move to @-",
                glyph::ARROW_CIRCLE_RIGHT,
                ContextAction::MoveBookmarkToParent(name.to_owned().into()),
            ),
            ContextMenuItem::new(
                "Push",
                glyph::ARROW_UP,
                ContextAction::PushBookmark(name.to_owned().into()),
            ),
        ];

        if !revset::is_trunk_bookmark(name) {
            items.push(ContextMenuItem::new(
                pull_request_label,
                glyph::ARROW_CIRCLE_RIGHT,
                ContextAction::OpenPRForBookmark(name.to_owned().into()),
            ));
        }
        if let Some(request) = self.bookmark_diff_request(name, cx) {
            items.push(ContextMenuItem::new(
                "Show Bookmark Diff",
                glyph::ARROWS_LEFT_RIGHT,
                ContextAction::ShowBookmarkDiff(request),
            ));
        }
        items.push(ContextMenuItem::new(
            "Copy Bookmark Name",
            glyph::COPY,
            ContextAction::CopyText(name.to_owned().into()),
        ));
        items
    }

    fn pull_request_menu_label(&self, cx: &App) -> String {
        self.vm
            .read(cx)
            .pull_request_host_name
            .as_ref()
            .map(|host| format!("Pull Request on {host}"))
            .unwrap_or_else(|| "Pull Request".to_owned())
    }

    pub(super) fn move_bookmark_to_parent(&mut self, name: SharedString, cx: &mut Context<Self>) {
        let bookmark = name.to_string();
        let task = self.vm.update(cx, |vm, cx| {
            vm.move_bookmark_to_parent(bookmark.clone(), cx)
        });
        cx.spawn(async move |this, cx| {
            if task.await.is_ok() {
                let _ = this.update(cx, move |view, cx| {
                    view.show_toast(format!("Moved {bookmark} to @-"), cx);
                });
            }
        })
        .detach();
    }

    pub(super) fn push_bookmark(&mut self, name: SharedString, cx: &mut Context<Self>) {
        let bookmark = name.to_string();
        let task = self
            .vm
            .update(cx, |vm, cx| vm.push_bookmark(bookmark.clone(), cx));
        cx.spawn(async move |this, cx| {
            if let Ok(message) = task.await {
                let _ = this.update(cx, move |view, cx| {
                    view.show_toast(push_status_message(&bookmark, &message), cx);
                });
            }
        })
        .detach();
    }

    pub(super) fn open_pr_for_bookmark(&mut self, name: SharedString, cx: &mut Context<Self>) {
        let Some(repo) = self.vm.read(cx).repo.clone() else {
            self.show_toast("Repository is not open", cx);
            return;
        };
        let bookmark = name.to_string();
        cx.spawn(async move |this, cx| {
            let url = cx
                .background_spawn(async move { repo.pull_request_open_url(&bookmark) })
                .await;
            let _ = this.update(cx, move |view, cx| {
                if let Some(url) = url {
                    cx.open_url(&url);
                } else {
                    view.show_toast(
                        "Couldn't determine a pull request URL — push the bookmark to a GitHub or Codeberg remote first.",
                        cx,
                    );
                }
            });
        })
        .detach();
    }

    fn bookmark_diff_request(&self, name: &str, cx: &App) -> Option<revset::BookmarkDiffRequest> {
        self.vm
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|change| change.bookmarks.iter().any(|bookmark| bookmark == name))
            .and_then(|change| revset::trunk_bookmark_diff_request(change, name))
    }
}

fn push_status_message(bookmark: &str, message: &str) -> String {
    message
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("Pushed {bookmark}"))
}
