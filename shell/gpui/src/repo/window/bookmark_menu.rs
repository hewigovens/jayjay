use gpui::{App, AppContext, Context, SharedString};
use jayjay_core::compare;
use jayjay_core::{BookmarkInfo, trunk};

use super::RepoWindow;
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
            jayjay_i18n::menu::resolve_conflict_set_to()
        } else {
            jayjay_i18n::menu::move_to()
        };
        let delete_label = if conflicted {
            jayjay_i18n::menu::remove_from_this_change()
        } else {
            jayjay_i18n::menu::delete_bookmark()
        };
        let bookmark = self
            .vm
            .read(cx)
            .graph
            .bookmarks
            .iter()
            .find(|candidate| candidate.name == name)
            .cloned();
        let is_tracking = bookmark
            .as_ref()
            .is_some_and(|info| info.is_tracking_remote);
        let has_local = bookmark
            .as_ref()
            .is_some_and(|info| info.has_local_target && !info.is_deleted);
        let untracked: Vec<String> = bookmark
            .as_ref()
            .map(|info| {
                info.available_remotes
                    .iter()
                    .filter(|remote| !info.tracked_remotes.contains(remote))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
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
                jayjay_i18n::menu::push(),
                glyph::ARROW_UP,
                ContextAction::PushBookmark(name.to_owned().into()),
            ),
        ];
        if is_tracking {
            items.push(ContextMenuItem::new(
                jayjay_i18n::menu::pull(),
                glyph::ARROW_DOWN,
                ContextAction::PullBookmark(name.to_owned().into()),
            ));
        }

        if !trunk::is_trunk_bookmark(name) {
            items.push(ContextMenuItem::new(
                pull_request_label,
                glyph::ARROW_CIRCLE_RIGHT,
                ContextAction::OpenPRForBookmark(name.to_owned().into()),
            ));
        }
        if let Some(request) = self.bookmark_diff_request(name, cx) {
            items.push(ContextMenuItem::new(
                jayjay_i18n::menu::show_bookmark_diff(),
                glyph::ARROWS_LEFT_RIGHT,
                ContextAction::ShowBookmarkDiff(request),
            ));
        }
        items.push(ContextMenuItem::new(
            jayjay_i18n::menu::copy_bookmark_name(),
            glyph::COPY,
            ContextAction::CopyText(name.to_owned().into()),
        ));
        if has_local {
            items.push(ContextMenuItem::new(
                jayjay_i18n::menu::rename_ellipsis(),
                glyph::PENCIL,
                ContextAction::RenameBookmark(name.to_owned().into()),
            ));
        }
        if has_local && !untracked.is_empty() {
            items.push(ContextMenuItem::submenu(
                jayjay_i18n::menu::track_remote(),
                glyph::GIT_BRANCH,
                untracked
                    .into_iter()
                    .map(|remote| {
                        ContextMenuItem::new(
                            remote.clone(),
                            glyph::GIT_BRANCH,
                            ContextAction::TrackBookmark {
                                name: name.to_owned(),
                                remote,
                            },
                        )
                    })
                    .collect(),
            ));
        }
        let can_delete = match rev {
            Some(_) => trunk::can_remove_bookmark_from_chip(name, conflicted),
            None => trunk::can_delete_bookmark(name, conflicted),
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
            .map(|host| jayjay_i18n::repo::pull_request_on(host))
            .unwrap_or_else(jayjay_i18n::menu::pull_request)
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
            "@" => jayjay_i18n::repo::resolved_bookmark_to_working_copy(&bookmark),
            "@-" => jayjay_i18n::repo::moved_bookmark_to_parent(&bookmark),
            _ => jayjay_i18n::repo::moved_bookmark(&bookmark),
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
                    jayjay_i18n::repo::removed_bookmark_from_change(&bookmark)
                } else {
                    jayjay_i18n::repo::deleted_bookmark(&bookmark)
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
            self.show_toast(jayjay_i18n::repo::repository_not_open(), cx);
            return;
        };
        let bookmark = name.to_string();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { repo.pull_request_open_url(&bookmark) })
                .await;
            let _ = this.update(cx, move |view, cx| match result {
                Ok(url) => crate::app::links::open_url(cx, &url),
                Err(error) => view.show_toast(crate::app::error_text(error), cx),
            });
        })
        .detach();
    }

    fn bookmark_diff_request(&self, name: &str, cx: &App) -> Option<compare::BookmarkDiffRequest> {
        self.vm
            .read(cx)
            .graph
            .changes
            .iter()
            .find(|change| change.bookmarks.iter().any(|bookmark| bookmark == name))
            .and_then(|change| compare::BookmarkDiffRequest::from_trunk(change, name))
    }
}
