use gpui::{Context, Focusable, IntoElement, ParentElement, Render, Styled, Window, div};

use super::super::bookmark_picker::render_bookmark_picker;
use super::super::detail::detail_pane;
use super::super::diff_edit::diff_edit_view;
use super::super::onboarding::onboarding_pane;
use super::super::rebase_confirmation::rebase_confirmation_overlay;
use super::super::repo_switcher::render_repo_switcher;
use super::super::sidebar::sidebar;
use super::super::status_bar::status_bar;
use super::super::{DragTarget, RepoWindow};
use super::layout::{file_column_wrapper, resize_handle};
use super::overlays::{error_overlay, text_modal_overlay, toast_overlay};
use super::repo_init::{repo_init_error_pane, repo_loading_pane};
use crate::app::theme::theme;
use crate::repo::toolbar::{BookmarkCounts, ToolbarActivity, ToolbarRepo};
#[cfg(not(target_os = "macos"))]
use crate::ui::app_menu::render_app_menu;
use crate::ui::context_menu::render_context_menu;

impl Render for RepoWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_diff_edit_change(cx);
        self.sync_editors(cx);
        // Cheap unless a note-affecting write happened (a single `stat` + small `Vec` compare); see `sync_review_notes`'s docs for why this can't just be a `mutate()`-only refresh.
        self.sync_review_notes(cx);
        let t = theme(cx).clone();
        let sidebar_width = self.layout.sidebar_width;
        let file_column_width = self.layout.file_column_width;
        let (toolbar_repo, bookmark_counts, bookmarks, workspaces, has_wc_changes, is_refreshing) = {
            let vm = self.vm.read(cx);
            let bookmarks = vm.graph.bookmarks.clone();
            let local_bookmarks = bookmarks
                .iter()
                .filter(|bookmark| !bookmark.is_deleted && bookmark.has_local_target)
                .collect::<Vec<_>>();
            let bookmark_counts = BookmarkCounts {
                total: local_bookmarks.len(),
                local_only: local_bookmarks
                    .iter()
                    .filter(|bookmark| !bookmark.is_tracking_remote)
                    .count(),
            };
            let workspaces = vm.graph.workspaces.clone();
            let toolbar_repo = ToolbarRepo {
                path: vm.repo_path.clone(),
                root_path: vm.repo_root_path.clone(),
                workspace: (workspaces.len() > 1)
                    .then(|| workspaces.iter().find(|workspace| workspace.is_current))
                    .flatten()
                    .map(|workspace| gpui::SharedString::from(workspace.name.clone())),
            };
            (
                toolbar_repo,
                bookmark_counts,
                bookmarks,
                workspaces,
                vm.loading.wc_changes,
                vm.loading.refresh_indicator,
            )
        };
        let is_fetching = self.sync_activity.fetching;
        let is_pushing = self.sync_activity.pushing;
        let init_error = {
            let vm = self.vm.read(cx);
            if vm.repo.is_none() {
                vm.error.clone()
            } else {
                None
            }
        };
        // Repo not open yet and no error → still opening async (see RepoViewModel::open_async).
        let opening_repo = {
            let vm = self.vm.read(cx);
            vm.repo.is_none() && vm.error.is_none()
        };
        let initializing_repo = self.vm.read(cx).loading.refreshing;
        let runtime_error = {
            let vm = self.vm.read(cx);
            if vm.repo.is_some() {
                vm.error.clone()
            } else {
                None
            }
        };

        let context_menu_overlay = self
            .context_menu
            .as_ref()
            .map(|state| render_context_menu(state, &t, &cx.entity()));
        #[cfg(not(target_os = "macos"))]
        let app_menu_overlay = self
            .app_menu
            .as_ref()
            .map(|state| render_app_menu(state, &t, &cx.entity(), cx));
        #[cfg(target_os = "macos")]
        let app_menu_overlay: Option<gpui::AnyElement> = None;
        let repo_switcher_overlay = self
            .repo_switcher
            .as_ref()
            .map(|state| render_repo_switcher(state, &workspaces, &t, &cx.entity()));
        let bookmark_picker_overlay = self
            .bookmark_picker
            .as_ref()
            .map(|state| render_bookmark_picker(state, &bookmarks, &t, &cx.entity()));

        let mut root = self.render_root(&t, cx);

        if let Some(onboarding) = self.onboarding.as_ref() {
            root = root.child(onboarding_pane(onboarding, &t, cx));
            if let Some(menu) = app_menu_overlay {
                root = root.child(menu);
            }
            return root.into_any_element();
        }

        if let Some(message) = init_error {
            root = root.child(repo_init_error_pane(
                toolbar_repo.path,
                message,
                initializing_repo,
                &t,
                cx,
            ));
            if let Some(menu) = app_menu_overlay {
                root = root.child(menu);
            }
            return root.into_any_element();
        }

        if opening_repo {
            root = root.child(repo_loading_pane(&t));
            if let Some(menu) = app_menu_overlay {
                root = root.child(menu);
            }
            return root.into_any_element();
        }

        let content = if self.diff_edit_active() {
            div()
                .flex()
                .flex_row()
                .flex_1()
                .min_h_0()
                .child(sidebar(self, &t, sidebar_width, cx))
                .child(resize_handle(DragTarget::Sidebar, &t, cx))
                .child(diff_edit_view(self, &t, cx))
        } else {
            div()
                .flex()
                .flex_row()
                .flex_1()
                .min_h_0()
                .child(sidebar(self, &t, sidebar_width, cx))
                .child(resize_handle(DragTarget::Sidebar, &t, cx))
                .child(file_column_wrapper(self, file_column_width, cx))
                .child(resize_handle(DragTarget::FileColumn, &t, cx))
                .child(detail_pane(self, &t, window, cx))
        };
        root = root
            .child(crate::repo::toolbar::toolbar(
                toolbar_repo,
                bookmark_counts,
                self.revset_filter_visible(),
                ToolbarActivity {
                    has_wc_changes,
                    is_refreshing,
                    is_fetching,
                    is_pushing,
                },
                cx,
            ))
            .child(content)
            .child(status_bar(self, &t, cx));

        if let Some(menu) = context_menu_overlay {
            root = root.child(menu);
        }
        if let Some(menu) = app_menu_overlay {
            root = root.child(menu);
        }
        if let Some(menu) = repo_switcher_overlay {
            root = root.child(menu);
        }
        if let Some(menu) = bookmark_picker_overlay {
            root = root.child(menu);
        }
        if self.diff_edit_take_pending_focus() {
            window.focus(&self.focus_handle, cx);
        }
        if self.text_modal.as_ref().is_some_and(|m| m.focus_pending) {
            let handle = self
                .text_modal
                .as_ref()
                .unwrap()
                .input
                .read(cx)
                .focus_handle(cx);
            window.focus(&handle, cx);
            if let Some(m) = self.text_modal.as_mut() {
                m.focus_pending = false;
            }
        }
        if let Some(modal) = self.text_modal.as_ref() {
            root = root.child(text_modal_overlay(modal, &t, cx));
        }
        if let Some(request) = self.pending_rebase.as_ref() {
            root = root.child(rebase_confirmation_overlay(request, &t, cx));
        }
        root = self.append_editor_overlays(root, &t, window, cx);
        if let Some(stacked_pr) = self.stacked_pr.as_ref() {
            root = root.child(super::super::stacked_pr_render::stacked_pr_overlay(
                stacked_pr, &t, cx,
            ));
        }
        if let Some(message) = self.feedback.toast.clone() {
            root = root.child(toast_overlay(message, &t));
        }
        if let Some(message) = runtime_error {
            root = root.child(error_overlay(message, &t, cx));
        }
        root.into_any_element()
    }
}
