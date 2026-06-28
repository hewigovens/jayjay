use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, AnyElement, ClickEvent, Context, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, ParentElement, SharedString, StatefulInteractiveElement, Styled,
    Transformation, Window, div, percentage, px, rgb, svg,
};

use crate::app::theme::Theme;
use crate::repo::window::RepoWindow;
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::{icon_label, text_tooltip, toolbar_button, toolbar_icon_button};
use crate::windows::settings::SettingsView;

#[derive(Clone, Copy)]
pub(super) enum RepoToolAction {
    Editor,
    Terminal,
}

#[derive(Clone, Copy)]
pub(super) enum SyncAction {
    FetchOrigin,
    PushDefault,
}

pub(super) fn bookmarks_button(
    count: usize,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let label = if count == 0 {
        SharedString::from("Bookmarks")
    } else {
        SharedString::from(format!("Bookmarks ({count})"))
    };
    div()
        .id(SharedString::from("tb-bookmarks"))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .px(px(10.))
        .py(px(4.))
        .rounded_sm()
        .bg(rgb(t.toolbar_button_bg))
        .text_size(px(11.))
        .text_color(rgb(t.fg_dim))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .active(|s| s.bg(rgb(t.selected_bg)))
        .tooltip(text_tooltip(label.clone()))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|view, ev: &MouseDownEvent, _w, cx| {
                view.open_bookmark_picker(ev.position, cx);
            }),
        )
        .child(icon_label(glyph::GIT_BRANCH, label, 14., t.fg_dim))
        .into_any_element()
}

pub(super) fn refresh_button(
    badge: bool,
    is_refreshing: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let mut content = div()
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .w(px(28.))
        .h(px(24.))
        .child(refresh_icon(is_refreshing, t));
    if badge {
        content = content.child(
            div()
                .absolute()
                .top(px(4.))
                .right(px(6.))
                .w(px(6.))
                .h(px(6.))
                .rounded_full()
                .bg(rgb(t.wc_accent)),
        );
    }
    toolbar_button("tb-refresh", "Refresh", t)
        .on_click(cx.listener(|view, _ev: &ClickEvent, _w, cx| {
            let vm = view.vm.clone();
            vm.update(cx, |vm, cx| vm.refresh(false, cx));
        }))
        .child(content)
        .into_any_element()
}

pub(super) fn sync_button(
    glyph_str: &'static str,
    id: &'static str,
    label: &'static str,
    action: SyncAction,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    toolbar_icon_button(id, glyph_str, label, t)
        .on_click(
            cx.listener(move |view, _ev: &ClickEvent, _w, cx| match action {
                SyncAction::FetchOrigin => view.git_fetch_origin(cx),
                SyncAction::PushDefault => view.git_push_default(cx),
            }),
        )
        .debug_selector(move || format!("toolbar-{label}"))
        .into_any_element()
}

pub(super) fn repo_tool_button(
    id: &'static str,
    glyph_str: &'static str,
    repo_path: SharedString,
    action: RepoToolAction,
    tooltip: SharedString,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    toolbar_icon_button(id, glyph_str, tooltip, t)
        .on_click(cx.listener(move |view, _ev: &ClickEvent, _w, cx| {
            let ok = match action {
                RepoToolAction::Editor => {
                    crate::app::tools::open_in_editor(repo_path.as_ref(), ".", cx)
                }
                RepoToolAction::Terminal => {
                    crate::app::tools::open_in_terminal(repo_path.as_ref(), cx)
                }
            };
            if !ok {
                view.show_toast(repo_tool_failure_message(action), cx);
            }
        }))
        .into_any_element()
}

fn repo_tool_failure_message(action: RepoToolAction) -> &'static str {
    match action {
        RepoToolAction::Editor => "Editor could not be opened",
        RepoToolAction::Terminal => "Terminal could not be opened",
    }
}

pub(super) fn settings_button(t: &Theme) -> AnyElement {
    toolbar_icon_button("tb-settings", glyph::GEAR, "Settings", t)
        .on_click(|_ev: &ClickEvent, _w: &mut Window, cx: &mut gpui::App| SettingsView::open(cx))
        .into_any_element()
}

pub(super) fn divider(t: &Theme) -> AnyElement {
    div()
        .w(px(1.))
        .h(px(20.))
        .bg(rgb(t.border))
        .into_any_element()
}

fn refresh_icon(is_refreshing: bool, t: &Theme) -> AnyElement {
    let icon = svg()
        .path(icons::REFRESH_CW_SVG)
        .w(px(14.))
        .h(px(14.))
        .text_color(rgb(t.fg_dim));
    if is_refreshing {
        icon.with_animation(
            "refresh-spinner",
            Animation::new(Duration::from_secs(1)).repeat(),
            |icon, delta| icon.with_transformation(Transformation::rotate(percentage(delta))),
        )
        .into_any_element()
    } else {
        icon.into_any_element()
    }
}
