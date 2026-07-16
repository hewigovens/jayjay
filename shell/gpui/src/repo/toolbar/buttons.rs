use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, AnyElement, ClickEvent, Context, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, ParentElement, SharedString, StatefulInteractiveElement, Styled,
    Transformation, Window, div, percentage, px, rgb, svg,
};

use crate::app::theme::Theme;
use crate::repo::window::RepoWindow;
use crate::ui::button_group::{self, GroupEdge, group_icon_item, group_item};
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::{TOOLBAR_BUTTON_HEIGHT, TOOLBAR_ICON_SIZE, icon_label};
use crate::windows::settings::SettingsView;

#[derive(Clone, Copy)]
enum RepoToolAction {
    Editor,
    Terminal,
}

#[derive(Clone, Copy)]
enum SyncAction {
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
        .h(px(TOOLBAR_BUTTON_HEIGHT))
        .px(px(12.))
        .rounded_full()
        .bg(rgb(t.toolbar_group_bg))
        .text_size(px(11.))
        .text_color(rgb(t.fg_dim))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .active(|s| s.bg(rgb(t.selected_bg)))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|view, ev: &MouseDownEvent, _w, cx| {
                view.open_bookmark_picker(ev.position, cx);
            }),
        )
        .child(icon_label(
            glyph::GIT_BRANCH,
            label,
            TOOLBAR_ICON_SIZE,
            t.fg_dim,
        ))
        .into_any_element()
}

fn revset_filter_button(
    active: bool,
    edge: GroupEdge,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let foreground = if active { t.toggle_active_fg } else { t.fg_dim };
    let mut button = group_item("tb-revset-filter", "Filter by revset", edge, t)
        .debug_selector(|| "toolbar-revset-filter".to_owned())
        .on_click(cx.listener(|view, _ev: &ClickEvent, window, cx| {
            view.toggle_revset_filter(window, cx);
        }));
    if active {
        button = button.bg(rgb(t.toggle_active_bg));
    }
    button
        .child(icons::icon(glyph::LIST, TOOLBAR_ICON_SIZE, foreground))
        .into_any_element()
}

pub(super) fn sync_cluster(
    revset_filter_active: bool,
    has_wc_changes: bool,
    is_refreshing: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    button_group::button_group(
        t,
        vec![
            revset_filter_button(revset_filter_active, GroupEdge::Leading, t, cx),
            refresh_button(has_wc_changes, is_refreshing, GroupEdge::Inner, t, cx),
            sync_button(
                glyph::ARROW_DOWN,
                "tb-pull",
                "Pull",
                SyncAction::FetchOrigin,
                GroupEdge::Inner,
                t,
                cx,
            ),
            sync_button(
                glyph::ARROW_UP,
                "tb-push",
                "Push",
                SyncAction::PushDefault,
                GroupEdge::Trailing,
                t,
                cx,
            ),
        ],
    )
    .id("sync-cluster")
    .debug_selector(|| "toolbar-sync-cluster".to_owned())
    .into_any_element()
}

pub(super) fn tools_cluster(
    repo_path: SharedString,
    open_editor_label: SharedString,
    open_terminal_label: SharedString,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    button_group::button_group(
        t,
        vec![
            repo_tool_button(
                repo_path.clone(),
                RepoToolAction::Editor,
                open_editor_label,
                GroupEdge::Leading,
                t,
                cx,
            ),
            repo_tool_button(
                repo_path,
                RepoToolAction::Terminal,
                open_terminal_label,
                GroupEdge::Inner,
                t,
                cx,
            ),
            settings_button(GroupEdge::Trailing, t),
        ],
    )
    .into_any_element()
}

fn refresh_button(
    badge: bool,
    is_refreshing: bool,
    edge: GroupEdge,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let mut content = div()
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .w_full()
        .h_full()
        .child(refresh_icon(is_refreshing, t));
    if badge {
        content = content.child(
            div()
                .absolute()
                .top(px(3.))
                .right(px(4.))
                .w(px(6.))
                .h(px(6.))
                .rounded_full()
                .bg(rgb(t.wc_accent)),
        );
    }
    group_item("tb-refresh", "Refresh", edge, t)
        .debug_selector(|| "toolbar-refresh".to_owned())
        .on_click(cx.listener(|view, _ev: &ClickEvent, _w, cx| {
            let vm = view.vm.clone();
            vm.update(cx, |vm, cx| vm.refresh(false, cx));
        }))
        .child(content)
        .into_any_element()
}

fn sync_button(
    glyph_str: &'static str,
    id: &'static str,
    label: &'static str,
    action: SyncAction,
    edge: GroupEdge,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    group_icon_item(id, glyph_str, label, edge, t)
        .on_click(
            cx.listener(move |view, _ev: &ClickEvent, _w, cx| match action {
                SyncAction::FetchOrigin => view.git_fetch_origin(cx),
                SyncAction::PushDefault => view.git_push_default(cx),
            }),
        )
        .debug_selector(move || format!("toolbar-{label}"))
        .into_any_element()
}

fn repo_tool_button(
    repo_path: SharedString,
    action: RepoToolAction,
    tooltip: SharedString,
    edge: GroupEdge,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let (id, glyph_str) = repo_tool_id_and_glyph(action);
    group_icon_item(id, glyph_str, tooltip, edge, t)
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

fn repo_tool_id_and_glyph(action: RepoToolAction) -> (&'static str, &'static str) {
    match action {
        RepoToolAction::Editor => ("tb-open-editor", glyph::BRACES),
        RepoToolAction::Terminal => ("tb-open-terminal", glyph::SQUARE_TERMINAL),
    }
}

fn settings_button(edge: GroupEdge, t: &Theme) -> AnyElement {
    group_icon_item("tb-settings", glyph::GEAR, "Settings", edge, t)
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
        .w(px(TOOLBAR_ICON_SIZE))
        .h(px(TOOLBAR_ICON_SIZE))
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
