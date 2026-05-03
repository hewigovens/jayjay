use gpui::{
    AnyElement, Context, CursorStyle, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ParentElement, Render, Styled, Window, div, px, rgb,
};

use super::detail::detail_pane;
use super::sidebar::sidebar;
use super::status_bar::status_bar;
use super::{DragTarget, LogView};
use crate::app::actions::{OpenCommandPalette, OpenFind, OpenSettings, Refresh};
use crate::app::theme::{Theme, theme};
use crate::diff::{FileColumnState, file_column};
use crate::ui::context_menu::render_context_menu;
use crate::windows::command_palette::CommandPalette;
use crate::windows::settings::SettingsView;

impl Render for LogView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme(cx).clone();
        let sidebar_width = self.sidebar_width;
        let file_column_width = self.file_column_width;
        let repo_path = self.vm.read(cx).repo_path.clone();
        let bookmark_count = self.vm.read(cx).graph.bookmarks.len();
        let has_wc_changes = self.vm.read(cx).loading.wc_changes;

        let context_menu_overlay = self
            .context_menu
            .as_ref()
            .map(|state| render_context_menu(state, &t, &cx.entity()));

        let mut root = div()
            .track_focus(&self.focus_handle)
            .key_context("LogView")
            .on_action(cx.listener(|_, _: &OpenSettings, _, cx| SettingsView::open(cx)))
            .on_action(cx.listener(|view, _: &OpenCommandPalette, _, cx| {
                let repo_path = view.vm.read(cx).repo_path.clone();
                CommandPalette::open(repo_path, cx);
            }))
            .on_action(cx.listener(|view, _: &OpenFind, _, cx| view.open_find(cx)))
            .on_action(
                cx.listener(|view, _: &crate::app::actions::CloseWindow, _, cx| {
                    if view.context_menu.is_some() {
                        view.close_context_menu(cx);
                    } else if view.find_query.is_some() {
                        view.close_find(cx);
                    }
                }),
            )
            .on_action(cx.listener(|view, _: &Refresh, _, cx| {
                let vm = view.vm.clone();
                let selected = vm.read(cx).selected;
                if let Some(ix) = selected {
                    vm.update(cx, |vm, cx| vm.select_change(ix, cx));
                }
            }))
            .on_key_down(cx.listener(|view, ev: &gpui::KeyDownEvent, _w, cx| {
                if view.handle_find_key(ev, cx) {
                    return;
                }
                view.handle_nav_key(ev, cx);
            }))
            .on_mouse_move(cx.listener(|view, ev: &MouseMoveEvent, _w, cx| {
                if view.drag.is_some() {
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
            .text_color(rgb(t.fg))
            .child(crate::repo::toolbar::toolbar(
                repo_path,
                bookmark_count,
                has_wc_changes,
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
        if let Some(message) = self.toast.clone() {
            root = root.child(toast_overlay(message, &t));
        }
        root
    }
}

/// Centered Xcode-style HUD overlay over the full LogView root.
fn toast_overlay(message: gpui::SharedString, _t: &Theme) -> AnyElement {
    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(10.))
                .px(px(32.))
                .py(px(24.))
                .min_w(px(220.))
                .rounded_lg()
                .bg(rgb(0x1c1c1e))
                .text_color(rgb(0xf2f2f7))
                .child(crate::ui::icons::icon(
                    crate::ui::icons::glyph::INFO,
                    40.,
                    0xf2f2f7,
                ))
                .child(
                    div()
                        .text_size(px(14.))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .child(message),
                ),
        )
        .into_any_element()
}

fn resize_handle(target: DragTarget, t: &Theme, cx: &mut Context<LogView>) -> AnyElement {
    div()
        .flex_none()
        .w(px(5.))
        .h_full()
        .cursor(CursorStyle::ResizeLeftRight)
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |view, ev: &MouseDownEvent, _w, cx| {
                view.start_drag(target, f32::from(ev.position.x), cx);
            }),
        )
        .child(div().w(px(1.)).h_full().ml(px(2.)).bg(rgb(t.border)))
        .into_any_element()
}

fn file_column_wrapper(view: &LogView, width: f32, cx: &mut Context<LogView>) -> AnyElement {
    let collapsed = view.collapsed_dirs.clone();
    let scroll = view.files_scroll.clone();
    let vm = view.vm.read(cx);
    let files = vm.files.as_ref().map(|v| v.as_slice().to_vec());
    let selected_file_ix = vm.selected_file_ix;
    let loading_files = vm.loading.files;
    let selected_change = vm.selected_change();
    let change_id = selected_change.map(|c| c.change_id.clone());
    let show_review = selected_change.map(|c| c.is_working_copy).unwrap_or(false);
    let reviewed_count = match (files.as_ref(), change_id.as_ref()) {
        (Some(fs), Some(cid)) => fs.iter().filter(|h| view.is_reviewed(cid, &h.path)).count(),
        _ => 0,
    };
    div()
        .w(px(width))
        .h_full()
        .child(file_column(
            FileColumnState {
                hunks: files.as_deref(),
                selected_ix: selected_file_ix,
                loading: loading_files,
                collapsed_dirs: &collapsed,
                scroll,
                change_id,
                reviewed_count,
                show_review,
            },
            cx,
        ))
        .into_any_element()
}
