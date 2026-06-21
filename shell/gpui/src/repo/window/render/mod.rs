mod layout;
mod overlays;
mod repo_init;

use gpui::{
    Context, Focusable, InteractiveElement, IntoElement, MouseButton, MouseMoveEvent, MouseUpEvent,
    ParentElement, Render, Styled, Window, div, rgb,
};

use super::detail::detail_pane;
use super::sidebar::sidebar;
use super::status_bar::status_bar;
use super::{DragTarget, RepoWindow};
use crate::app::actions::{CopyDiffSelection, OpenCommandPalette, OpenFind, OpenSettings, Refresh};
use crate::app::theme::theme;
use crate::ui::context_menu::render_context_menu;
use crate::windows::command_palette::CommandPalette;
use crate::windows::settings::SettingsView;
use layout::{file_column_wrapper, resize_handle};
use overlays::{error_overlay, text_modal_overlay, toast_overlay};
use repo_init::{repo_init_error_pane, repo_loading_pane};

impl Render for RepoWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx).clone();
        let sidebar_width = self.layout.sidebar_width;
        let file_column_width = self.layout.file_column_width;
        let repo_path = self.vm.read(cx).repo_path.clone();
        let bookmark_count = self.vm.read(cx).graph.bookmarks.len();
        let (has_wc_changes, is_refreshing) = {
            let vm = self.vm.read(cx);
            (vm.loading.wc_changes, vm.loading.refresh_indicator)
        };
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

        let mut root = div()
            .track_focus(&self.focus_handle)
            .key_context("RepoWindow")
            .on_action(cx.listener(|_, _: &OpenSettings, _, cx| SettingsView::open(cx)))
            .on_action(cx.listener(|view, _: &OpenCommandPalette, _, cx| {
                let repo_path = view.vm.read(cx).repo_path.clone();
                CommandPalette::open(repo_path, Some(cx.entity()), cx);
            }))
            .on_action(cx.listener(|view, _: &OpenFind, _, cx| view.open_find(cx)))
            .on_action(
                cx.listener(|view, _: &CopyDiffSelection, _, cx| view.copy_diff_selection(cx)),
            )
            .on_action(
                cx.listener(|view, _: &crate::app::actions::Dismiss, _, cx| {
                    view.dismiss_overlay(cx);
                }),
            )
            .on_action(
                cx.listener(|view, _: &crate::app::actions::CloseWindow, window, cx| {
                    // cmd-w: close any open overlay first, otherwise close the window.
                    if !view.dismiss_overlay(cx) {
                        window.remove_window();
                    }
                }),
            )
            .on_action(cx.listener(|view, _: &Refresh, _, cx| {
                let vm = view.vm.clone();
                vm.update(cx, |vm, cx| vm.refresh(false, cx));
            }))
            .on_key_down(cx.listener(|view, ev: &gpui::KeyDownEvent, window, cx| {
                if view.is_text_input_focused(window, cx) {
                    return;
                }
                if view.handle_find_key(ev, cx) {
                    return;
                }
                view.handle_nav_key(ev, cx);
            }))
            .on_mouse_move(cx.listener(|view, ev: &MouseMoveEvent, _w, cx| {
                if view.layout.drag.is_some() {
                    view.drag_to(f32::from(ev.position.x), f32::from(ev.position.y), cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|view, _: &MouseUpEvent, _w, cx| {
                    view.end_drag(cx);
                }),
            )
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(t.detail_bg))
            .text_color(rgb(t.fg));

        if let Some(message) = init_error {
            return root
                .child(repo_init_error_pane(
                    repo_path,
                    message,
                    initializing_repo,
                    &t,
                    cx,
                ))
                .into_any_element();
        }

        if opening_repo {
            return root.child(repo_loading_pane(&t)).into_any_element();
        }

        root = root
            .child(crate::repo::toolbar::toolbar(
                repo_path,
                bookmark_count,
                has_wc_changes,
                is_refreshing,
                cx,
            ))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_h_0()
                    .child(sidebar(self, &t, sidebar_width, cx))
                    .child(resize_handle(DragTarget::Sidebar, &t, cx))
                    .child(file_column_wrapper(self, file_column_width, cx))
                    .child(resize_handle(DragTarget::FileColumn, &t, cx))
                    .child(detail_pane(self, &t, cx)),
            )
            .child(status_bar(self, &t, cx));

        if let Some(menu) = context_menu_overlay {
            root = root.child(menu);
        }
        if self.text_modal.as_ref().is_some_and(|m| m.focus_pending) {
            let handle = self.text_modal.as_ref().unwrap().input.read(cx).focus_handle(cx);
            window.focus(&handle, cx);
            if let Some(m) = self.text_modal.as_mut() {
                m.focus_pending = false;
            }
        }
        if let Some(modal) = self.text_modal.as_ref() {
            root = root.child(text_modal_overlay(modal, &t, cx));
        }
        if let Some(message) = self.feedback.toast.clone() {
            root = root.child(toast_overlay(message));
        }
        if let Some(message) = runtime_error {
            root = root.child(error_overlay(message, &t, cx));
        }
        root.into_any_element()
    }
}

impl RepoWindow {
    /// Dismiss the topmost open overlay; returns `true` when one was dismissed so cmd-w can
    /// fall through to closing the window only when nothing was open.
    fn dismiss_overlay(&mut self, cx: &mut Context<Self>) -> bool {
        let has_runtime_error = {
            let vm = self.vm.read(cx);
            vm.repo.is_some() && vm.error.is_some()
        };
        if has_runtime_error {
            self.vm.update(cx, |vm, cx| {
                vm.clear_error();
                cx.notify();
            });
        } else if self.text_modal.is_some() {
            self.close_text_modal(cx);
        } else if self.context_menu.is_some() {
            self.close_context_menu(cx);
        } else if self.find.query.is_some() {
            self.close_find(cx);
        } else {
            return false;
        }
        true
    }

    fn is_text_input_focused(&self, window: &Window, cx: &gpui::App) -> bool {
        if self
            .summary_input
            .read(cx)
            .focus_handle(cx)
            .is_focused(window)
            || self
                .description_input
                .read(cx)
                .focus_handle(cx)
                .is_focused(window)
        {
            return true;
        }
        self.text_modal
            .as_ref()
            .is_some_and(|modal| modal.input.read(cx).focus_handle(cx).is_focused(window))
    }
}
