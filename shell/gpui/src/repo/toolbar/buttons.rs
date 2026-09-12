use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, AnyElement, ClickEvent, Context, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, ParentElement, SharedString, StatefulInteractiveElement, Styled,
    Transformation, Window, div, percentage, point, px, rgb, svg,
};

use crate::app::theme::{Theme, ui_font_size};
use crate::repo::toolbar::{BookmarkCounts, ToolbarActivity};
use crate::repo::window::{FocusStop, RepoWindow, focus_ring};
use crate::ui::button_group::{self, GroupEdge, group_icon_item, group_item};
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::{TOOLBAR_BUTTON_HEIGHT, TOOLBAR_ICON_SIZE, icon_label};
use crate::windows::settings::SettingsView;

#[derive(Clone, Copy)]
enum RepoToolAction {
    Editor,
    Terminal,
}

impl RepoToolAction {
    fn focus_stop(self) -> FocusStop {
        match self {
            Self::Editor => FocusStop::Editor,
            Self::Terminal => FocusStop::Terminal,
        }
    }
}

#[derive(Clone, Copy)]
enum SyncAction {
    FetchOrigin,
    PushDefault,
}

impl SyncAction {
    fn focus_stop(self) -> FocusStop {
        match self {
            Self::FetchOrigin => FocusStop::Pull,
            Self::PushDefault => FocusStop::Push,
        }
    }

    fn icon_path(self) -> &'static str {
        match self {
            Self::FetchOrigin => icons::ARROW_DOWN_SVG,
            Self::PushDefault => icons::ARROW_UP_SVG,
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::FetchOrigin => "tb-pull",
            Self::PushDefault => "tb-push",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::FetchOrigin => "Pull",
            Self::PushDefault => "Push",
        }
    }

    fn direction(self) -> f32 {
        match self {
            Self::FetchOrigin => 1.,
            Self::PushDefault => -1.,
        }
    }
}

pub(super) fn bookmarks_button(
    counts: BookmarkCounts,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let BookmarkCounts {
        total: count,
        local_only,
    } = counts;
    let label = if count == 0 {
        SharedString::from("Bookmarks")
    } else if local_only > 0 {
        SharedString::from(format!("Bookmarks ({count}, {local_only} local)"))
    } else {
        SharedString::from(format!("Bookmarks ({count})"))
    };
    div()
        .id(SharedString::from("tb-bookmarks"))
        .debug_selector(move || format!("toolbar-bookmarks-{count}"))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .h(px(TOOLBAR_BUTTON_HEIGHT))
        .px(px(12.))
        .rounded_full()
        .bg(rgb(t.toolbar_group_bg))
        .text_size(ui_font_size(11.))
        .text_color(rgb(t.fg_dim))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .active(|s| s.bg(rgb(t.selected_bg)))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|view, ev: &MouseDownEvent, window, cx| {
                view.focus_handle.focus(window, cx);
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
    focused: Option<FocusStop>,
    edge: GroupEdge,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let foreground = if active { t.toggle_active_fg } else { t.fg_dim };
    let mut button = focus_ring(
        group_item("tb-revset-filter", "Filter by revset", edge, t),
        focused == Some(FocusStop::RevsetFilter),
        t,
    )
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
    activity: ToolbarActivity,
    focused: Option<FocusStop>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    button_group::button_group(
        t,
        vec![
            revset_filter_button(revset_filter_active, focused, GroupEdge::Leading, t, cx),
            refresh_button(activity.is_refreshing, focused, GroupEdge::Inner, t, cx),
            sync_button(
                SyncAction::FetchOrigin,
                activity.is_fetching,
                focused,
                GroupEdge::Inner,
                t,
                cx,
            ),
            sync_button(
                SyncAction::PushDefault,
                activity.is_pushing,
                focused,
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
    open_editor_label: SharedString,
    open_terminal_label: SharedString,
    focused: Option<FocusStop>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    button_group::button_group(
        t,
        vec![
            repo_tool_button(
                RepoToolAction::Editor,
                open_editor_label,
                focused,
                GroupEdge::Leading,
                t,
                cx,
            ),
            repo_tool_button(
                RepoToolAction::Terminal,
                open_terminal_label,
                focused,
                GroupEdge::Inner,
                t,
                cx,
            ),
            settings_button(focused, GroupEdge::Trailing, t),
        ],
    )
    .into_any_element()
}

fn refresh_button(
    is_refreshing: bool,
    focused: Option<FocusStop>,
    edge: GroupEdge,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let content = div()
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .w_full()
        .h_full()
        .child(refresh_icon(is_refreshing, t));
    focus_ring(
        group_item("tb-refresh", "Refresh", edge, t),
        focused == Some(FocusStop::Refresh),
        t,
    )
    .debug_selector(|| "toolbar-refresh".to_owned())
    .on_click(cx.listener(|view, _ev: &ClickEvent, _w, cx| {
        let vm = view.vm.clone();
        vm.update(cx, |vm, cx| vm.refresh(false, cx));
    }))
    .child(content)
    .into_any_element()
}

fn sync_button(
    action: SyncAction,
    animating: bool,
    focused: Option<FocusStop>,
    edge: GroupEdge,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let id = action.id();
    let label = action.label();
    focus_ring(
        group_item(id, label, edge, t),
        focused == Some(action.focus_stop()),
        t,
    )
    .on_click(
        cx.listener(move |view, _ev: &ClickEvent, _w, cx| match action {
            SyncAction::FetchOrigin => view.git_fetch_origin(cx),
            SyncAction::PushDefault => view.git_push_default(cx),
        }),
    )
    .debug_selector(move || format!("toolbar-{label}"))
    .child(sync_icon(
        action.icon_path(),
        id,
        animating,
        action.direction(),
        t,
    ))
    .into_any_element()
}

fn sync_icon(
    path: &'static str,
    animation_id: &'static str,
    animating: bool,
    direction: f32,
    t: &Theme,
) -> AnyElement {
    let arrow_size = TOOLBAR_ICON_SIZE / 2.;
    let arrow_inset = (TOOLBAR_ICON_SIZE - arrow_size) / 2.;
    let circle = svg()
        .path(icons::CIRCLE_SVG)
        .w(px(TOOLBAR_ICON_SIZE))
        .h(px(TOOLBAR_ICON_SIZE))
        .text_color(rgb(t.fg_dim));
    let arrow = svg().path(path).size_full().text_color(rgb(t.fg_dim));
    let arrow = if !animating {
        arrow.into_any_element()
    } else {
        arrow
            .with_animation(
                animation_id,
                Animation::new(Duration::from_millis(700)).repeat(),
                move |arrow, delta| {
                    let offset = direction * (-2. + 4. * delta);
                    let opacity = (delta.min(1. - delta) * 5.).min(1.);
                    arrow
                        .opacity(opacity)
                        .with_transformation(Transformation::translate(point(px(0.), px(offset))))
                },
            )
            .into_any_element()
    };
    let arrow = div()
        .absolute()
        .top(px(arrow_inset))
        .left(px(arrow_inset))
        .w(px(arrow_size))
        .h(px(arrow_size))
        .debug_selector(move || format!("sync-arrow-{animation_id}"))
        .child(arrow);
    div()
        .relative()
        .flex_none()
        .w(px(TOOLBAR_ICON_SIZE))
        .h(px(TOOLBAR_ICON_SIZE))
        .debug_selector(move || format!("sync-icon-{animation_id}"))
        .child(circle)
        .child(arrow)
        .into_any_element()
}

fn repo_tool_button(
    action: RepoToolAction,
    tooltip: SharedString,
    focused: Option<FocusStop>,
    edge: GroupEdge,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let (id, glyph_str) = repo_tool_id_and_glyph(action);
    focus_ring(
        group_icon_item(id, glyph_str, tooltip, edge, t),
        focused == Some(action.focus_stop()),
        t,
    )
    .on_click(
        cx.listener(move |view, _ev: &ClickEvent, _w, cx| match action {
            RepoToolAction::Editor => view.open_repo_in_editor(cx),
            RepoToolAction::Terminal => view.open_repo_in_terminal(cx),
        }),
    )
    .into_any_element()
}

fn repo_tool_id_and_glyph(action: RepoToolAction) -> (&'static str, &'static str) {
    match action {
        RepoToolAction::Editor => ("tb-open-editor", glyph::BRACES),
        RepoToolAction::Terminal => ("tb-open-terminal", glyph::SQUARE_TERMINAL),
    }
}

fn settings_button(focused: Option<FocusStop>, edge: GroupEdge, t: &Theme) -> AnyElement {
    focus_ring(
        group_icon_item("tb-settings", glyph::GEAR, "Settings", edge, t),
        focused == Some(FocusStop::Settings),
        t,
    )
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
