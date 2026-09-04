use gpui::{App, AppContext, Context, SharedString};
use jayjay_core::BookmarkInfo;

use super::RepoWindow;
use crate::repo::revset;
use crate::ui::context_menu::{ContextAction, ContextMenuItem};
use crate::ui::icons::glyph;

impl RepoWindow {
    /// `rev` is the change a chip was clicked on; the picker passes `None` to act on the bookmark as a whole.
    pub(super) fn build_bookmark_menu(
        &self,
        name: &str,
        rev: Option<&str>,
        cx: &App,
    ) -> Vec<ContextMenuItem> {
        let pull_request_label = self.pull_request_menu_label(cx);
        let conflicted = self.bookmark_is_conflicted(name, cx);
        let move_label = if conflicted {
            "Resolve conflict (set to @)"
        } else {
            "Move to @-"
        };
        let delete_label = if conflicted {
            "Remove from This Change"
        } else {
            "Delete Bookmark"
        };
        let mut items = vec![
            ContextMenuItem::new(
                move_label,
                glyph::ARROW_CIRCLE_RIGHT,
                ContextAction::MoveBookmark {
                    name: name.to_owned().into(),
                    to_rev: if conflicted { "@".into() } else { "@-".into() },
                },
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
        let can_delete = match rev {
            Some(_) => revset::can_remove_bookmark_from_chip(name, conflicted),
            None => !conflicted && !revset::is_trunk_bookmark(name),
        };
        if can_delete {
            items.push(ContextMenuItem::new(
                delete_label,
                glyph::X_CIRCLE,
                ContextAction::DeleteBookmark {
                    name: name.to_owned().into(),
                    rev: rev.map(|rev| rev.to_owned().into()),
                },
            ));
        }
        items
    }

    fn pull_request_menu_label(&self, cx: &App) -> String {
        self.vm
            .read(cx)
            .pr_host_name
            .as_ref()
            .map(|host| format!("Pull Request on {host}"))
            .unwrap_or_else(|| "Pull Request".to_owned())
    }

    pub(super) fn bookmark_is_conflicted(&self, name: &str, cx: &App) -> bool {
        BookmarkInfo::is_conflicted_name(&self.vm.read(cx).graph.bookmarks, name)
    }

    pub(super) fn move_bookmark(
        &mut self,
        name: SharedString,
        to_rev: SharedString,
        cx: &mut Context<Self>,
    ) {
        let bookmark = name.to_string();
        let dest = to_rev.to_string();
        let message = match dest.as_str() {
            "@" => format!("Resolved {bookmark} to @"),
            "@-" => format!("Moved {bookmark} to @-"),
            _ => format!("Moved {bookmark}"),
        };
        self.move_bookmark_to_rev(bookmark, dest, message, cx);
    }

    pub(super) fn move_bookmark_to_rev(
        &mut self,
        bookmark: String,
        dest: String,
        success_message: String,
        cx: &mut Context<Self>,
    ) {
        let was_tracking = self
            .vm
            .read(cx)
            .graph
            .bookmarks
            .iter()
            .any(|candidate| candidate.name == bookmark && candidate.is_tracking_remote);
        let task = self
            .vm
            .update(cx, |vm, cx| vm.move_bookmark(bookmark.clone(), dest, cx));
        cx.spawn(async move |this, cx| {
            if task.await.is_ok() {
                let _ = this.update(cx, move |view, cx| {
                    if was_tracking {
                        view.feedback.pending_push_bookmark = Some(bookmark.into());
                    }
                    view.show_toast(success_message, cx);
                });
            }
        })
        .detach();
    }

    pub(super) fn delete_bookmark(
        &mut self,
        name: SharedString,
        rev: Option<SharedString>,
        cx: &mut Context<Self>,
    ) {
        let bookmark = name.to_string();
        let conflicted = self.bookmark_is_conflicted(&bookmark, cx);
        let task = self.vm.update(cx, |vm, cx| match rev {
            Some(rev) => vm.remove_bookmark_from_rev(bookmark.clone(), rev.to_string(), cx),
            None => vm.delete_bookmark(bookmark.clone(), cx),
        });
        cx.spawn(async move |this, cx| {
            if task.await.is_ok() {
                let message = if conflicted {
                    format!("Removed {bookmark} from this change")
                } else {
                    format!("Deleted bookmark {bookmark}")
                };
                let _ = this.update(cx, move |view, cx| {
                    view.show_toast(message, cx);
                });
            }
        })
        .detach();
    }

    pub(super) fn push_bookmark(&mut self, name: SharedString, cx: &mut Context<Self>) {
        self.git_push_bookmark(name.to_string(), cx);
    }

    pub(super) fn open_pr_for_bookmark(&mut self, name: SharedString, cx: &mut Context<Self>) {
        let Some(repo) = self.vm.read(cx).repo.clone() else {
            self.show_toast("Repository is not open", cx);
            return;
        };
        let bookmark = name.to_string();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { repo.pull_request_open_url(&bookmark) })
                .await;
            let _ = this.update(cx, move |view, cx| match result {
                Ok(url) => crate::app::links::open_url(cx, &url),
                Err(error) => view.show_toast(error.to_string(), cx),
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
