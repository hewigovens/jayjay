use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, AnyElement, App, ClickEvent, Context, InteractiveElement,
    IntoElement, ParentElement, SharedString, StatefulInteractiveElement, Styled, Transformation,
    div, percentage, point, px, rgb, rgba, svg,
};

use crate::app::actions::{ToggleOperationLog, ToggleSidebar};
use crate::app::key_caps::KeyCaps;
use crate::app::theme::{Theme, with_alpha};
use crate::repo::toolbar::{ToolButton, ToolbarActivity};
use crate::repo::view_model::RefreshMode;
use crate::repo::window::{FocusStop, RepoWindow, focus_ring, operation_log_title};
use crate::ui::button_group::{self, GroupEdge, group_item};
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::TOOLBAR_ICON_SIZE;

const SYNC_ICON_HEIGHT: f32 = TOOLBAR_ICON_SIZE * 0.8;
const SYNC_ICON_ASPECT: f32 = 19.3 / 16.;
const MARK_TILE: f32 = TOOLBAR_ICON_SIZE * 1.125;
const MARK_SIZE: f32 = TOOLBAR_ICON_SIZE * 0.78;

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

    fn id(self) -> &'static str {
        match self {
            Self::Editor => "tb-open-editor",
            Self::Terminal => "tb-open-terminal",
        }
    }

    fn fallback_glyph(self) -> &'static str {
        match self {
            Self::Editor => glyph::BRACES,
            Self::Terminal => glyph::TERMINAL,
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
            Self::FetchOrigin => icons::SYNC_PULL_SVG,
            Self::PushDefault => icons::SYNC_PUSH_SVG,
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::FetchOrigin => "tb-pull",
            Self::PushDefault => "tb-push",
        }
    }

    fn label(self) -> String {
        match self {
            Self::FetchOrigin => jayjay_i18n::menu::pull(),
            Self::PushDefault => jayjay_i18n::menu::push(),
        }
    }

    fn direction(self) -> f32 {
        match self {
            Self::FetchOrigin => 1.,
            Self::PushDefault => -1.,
        }
    }
}

fn sidebar_toggle_button(
    sidebar_hidden: bool,
    focused: Option<FocusStop>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let title = if sidebar_hidden {
        jayjay_i18n::menu::show_sidebar()
    } else {
        jayjay_i18n::menu::hide_sidebar()
    };
    let tooltip = KeyCaps::tooltip(title, &ToggleSidebar, cx);
    focus_ring(
        group_item("tb-sidebar-toggle", tooltip, GroupEdge::Leading, t).child(
            svg()
                .path(icons::SIDEBAR_LEADING_SVG)
                .size(px(TOOLBAR_ICON_SIZE))
                .text_color(rgb(t.fg_dim)),
        ),
        focused == Some(FocusStop::SidebarToggle),
        t,
    )
    .debug_selector(|| "toolbar-sidebar-toggle".to_owned())
    .on_click(cx.listener(|view, _ev: &ClickEvent, _w, cx| {
        view.toggle_sidebar(cx);
    }))
    .into_any_element()
}

pub(super) fn sync_cluster(
    sidebar_hidden: bool,
    activity: ToolbarActivity,
    focused: Option<FocusStop>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    button_group::button_group(
        t,
        vec![
            sidebar_toggle_button(sidebar_hidden, focused, t, cx),
            refresh_button(
                activity.is_refreshing,
                activity.refresh,
                focused,
                GroupEdge::Inner,
                t,
                cx,
            ),
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
    editor: ToolButton,
    terminal: ToolButton,
    focused: Option<FocusStop>,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    button_group::button_group(
        t,
        vec![
            repo_tool_button(
                RepoToolAction::Editor,
                editor,
                focused,
                GroupEdge::Leading,
                t,
                cx,
            ),
            repo_tool_button(
                RepoToolAction::Terminal,
                terminal,
                focused,
                GroupEdge::Trailing,
                t,
                cx,
            ),
        ],
    )
    .into_any_element()
}

pub(super) fn operation_log_toggle(
    shown: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let tooltip = KeyCaps::tooltip(operation_log_title(shown), &ToggleOperationLog, cx);
    button_group::button_group(
        t,
        vec![
            group_item("tb-operation-log", tooltip, GroupEdge::Leading, t)
                .rounded_full()
                .debug_selector(|| "toolbar-operation-log".to_owned())
                .on_click(cx.listener(|view, _ev: &ClickEvent, _w, cx| {
                    view.toggle_operation_log(cx);
                }))
                .child(icons::icon(glyph::HISTORY, TOOLBAR_ICON_SIZE, t.fg_dim))
                .into_any_element(),
        ],
    )
    .into_any_element()
}

fn refresh_button(
    is_refreshing: bool,
    mode: RefreshMode,
    focused: Option<FocusStop>,
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
    if mode.shows_badge() {
        content = content.child(
            div()
                .absolute()
                .top(px(4.))
                .right(px(5.))
                .size(px(7.))
                .rounded_full()
                .bg(rgb(t.warning_fg))
                .debug_selector(|| "toolbar-refresh-stale-badge".to_owned()),
        );
    }
    focus_ring(
        group_item("tb-refresh", mode.tooltip(), edge, t),
        focused == Some(FocusStop::Refresh),
        t,
    )
    .debug_selector(|| "toolbar-refresh".to_owned())
    .on_click(cx.listener(|view, _ev: &ClickEvent, _w, cx| {
        view.vm.update(cx, |vm, cx| vm.run_refresh(cx));
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
            SyncAction::FetchOrigin => view.pull_or_cancel(cx),
            SyncAction::PushDefault => view.push_or_cancel(cx),
        }),
    )
    .debug_selector(move || action.id().to_owned())
    .child(sync_icon(
        action.icon_path(),
        id,
        animating,
        action.direction(),
        t,
        cx,
    ))
    .into_any_element()
}

fn sync_icon(
    path: &'static str,
    animation_id: &'static str,
    animating: bool,
    direction: f32,
    t: &Theme,
    cx: &App,
) -> AnyElement {
    let icon = svg()
        .path(path)
        .w(px(SYNC_ICON_HEIGHT * SYNC_ICON_ASPECT))
        .h(px(SYNC_ICON_HEIGHT))
        .text_color(rgb(t.fg_dim));
    if !animating || cx.reduce_motion() {
        return icon.into_any_element();
    }
    icon.with_animation(
        animation_id,
        Animation::new(Duration::from_millis(900)).repeat(),
        move |icon, delta| {
            let nudge = 1.5 * (std::f32::consts::PI * (delta / 0.78).min(1.)).sin();
            icon.with_transformation(Transformation::translate(point(
                px(0.),
                px(direction * nudge),
            )))
        },
    )
    .into_any_element()
}

fn repo_tool_button(
    action: RepoToolAction,
    button: ToolButton,
    focused: Option<FocusStop>,
    edge: GroupEdge,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let selector = format!(
        "toolbar-mark-{}",
        button.mark.map_or("generic", |mark| mark.name)
    );
    let mark = match button.mark {
        Some(mark) => svg()
            .path(SharedString::from(format!(
                "{}{}",
                icons::MARK_PREFIX,
                mark.name
            )))
            .size(px(MARK_SIZE * mark.scale))
            .text_color(rgb(t.fg_dim))
            .into_any_element(),
        None => icons::icon(action.fallback_glyph(), MARK_SIZE, t.fg_dim).into_any_element(),
    };
    focus_ring(
        group_item(action.id(), button.tooltip, edge, t).child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(MARK_TILE))
                .rounded(px(MARK_TILE * 0.27))
                .bg(rgba(with_alpha(t.fg, 0x1a)))
                .debug_selector(move || selector.clone())
                .child(mark),
        ),
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

fn refresh_icon(is_refreshing: bool, t: &Theme) -> AnyElement {
    let icon = svg()
        .path(icons::REFRESH_SVG)
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
