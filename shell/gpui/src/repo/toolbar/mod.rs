mod buttons;

use gpui::{
    AnyElement, Context, Div, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, SharedString, Stateful, Styled, div, px, rgb,
};

use crate::app::theme::{theme, ui_font_size};
use crate::app::{config, repositories, tools};
use crate::platform::{TOOLBAR_HEIGHT, TOOLBAR_LEADING_INSET};
use crate::repo::view_model::RefreshMode;
use crate::repo::window::{
    FocusStop, PanelBoundsSlot, RepoWindow, picker_opener, picker_opener_bounds,
};
use crate::ui::icons;
use crate::ui::primitives::{TOOLBAR_BUTTON_HEIGHT, TOOLBAR_BUTTON_WIDTH};
use jayjay_core::ToolMark;

pub(crate) struct ToolbarActivity {
    pub(crate) is_refreshing: bool,
    pub(crate) refresh: RefreshMode,
    pub(crate) is_fetching: bool,
    pub(crate) is_pushing: bool,
}

pub(crate) struct ToolButton {
    pub(crate) tooltip: SharedString,
    pub(crate) mark: Option<ToolMark>,
}

pub(crate) struct ToolbarRepo {
    pub(crate) path: SharedString,
    pub(crate) root_path: SharedString,
    /// Shown after the repository name only when the repo has more than one workspace.
    pub(crate) workspace: Option<SharedString>,
}

pub(crate) const TOOLBAR_TRAILING_INSET: f32 = 12.;
const TOOLBAR_GAP: f32 = 6.;
pub(crate) const LEADING_TOGGLE_ROOM: f32 =
    TOOLBAR_LEADING_INSET + TOOLBAR_BUTTON_WIDTH + TOOLBAR_GAP;
pub(crate) const TRAILING_TOGGLE_ROOM: f32 =
    TOOLBAR_GAP + TOOLBAR_BUTTON_WIDTH + TOOLBAR_TRAILING_INSET;
/// Aligns the repository name's text with the change header's.
pub(crate) const TITLE_INSET: f32 = crate::diff::DETAIL_INSET - TITLE_PADDING;
const TITLE_PADDING: f32 = 10.;

pub(crate) use buttons::{operation_log_toggle, sidebar_toggle_button};

/// Zooms the window on a double click in its empty space, as a title bar does.
fn strip(id: &'static str, cx: &mut Context<RepoWindow>) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .flex_row()
        .flex_none()
        .items_center()
        .h(px(TOOLBAR_HEIGHT))
        .gap(px(TOOLBAR_GAP))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|view, ev: &MouseDownEvent, window, cx| {
                let on_bar = view
                    .revset_bar_bounds
                    .get()
                    .is_some_and(|bounds| bounds.contains(&ev.position));
                if ev.click_count == 2 && !on_bar && !window.default_prevented() {
                    window.zoom_window();
                }
                // Title-bar clicks take no focus of their own, so an open revset edit would otherwise survive them.
                if view.revset_editor.is_some() && !on_bar {
                    view.focus_handle.focus(window, cx);
                }
            }),
        )
}

/// Refresh, pull and push after the sidebar toggle's room, so they leave with the sidebar.
pub(crate) fn sidebar_strip(
    activity: ToolbarActivity,
    focused: Option<FocusStop>,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let t = theme(cx).clone();
    strip("toolbar-sidebar-strip", cx)
        .w_full()
        .pl(px(LEADING_TOGGLE_ROOM))
        .bg(rgb(t.sidebar_bg))
        .child(buttons::sync_cluster(activity, focused, &t, cx))
        .into_any_element()
}

pub(crate) fn operation_log_strip(cx: &mut Context<RepoWindow>) -> AnyElement {
    let t = theme(cx).clone();
    strip("toolbar-operation-log-strip", cx)
        .w_full()
        .bg(rgb(t.sidebar_bg))
        .into_any_element()
}

/// `leading` and `trailing` keep room for the column toggles.
pub(crate) fn toolbar(
    repo: ToolbarRepo,
    repo_switcher_bounds: PanelBoundsSlot,
    leading: AnyElement,
    revset_bar: AnyElement,
    trailing: AnyElement,
    focused: Option<FocusStop>,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let t = theme(cx).clone();

    let repo_name = repositories::repository_name(&repo.root_path);
    let repo_name_selector = format!("repo-title-repository-{repo_name}");
    let config = config::current(cx);
    let editor = ToolButton {
        tooltip: tools::open_in_editor_label(cx).into(),
        mark: ToolMark::editor(&config.tools.external_editor),
    };
    let terminal = ToolButton {
        tooltip: tools::open_in_terminal_label(cx).into(),
        mark: ToolMark::terminal(&config.tools.terminal),
    };

    strip("toolbar", cx)
        .w_full()
        .min_w_0()
        .bg(rgb(t.toolbar_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(div().flex_none().mr(px(-TOOLBAR_GAP)).child(leading))
        .child(picker_opener(
            div()
                .id("repo-switcher-button")
                .debug_selector(|| "repo-switcher-button".to_owned())
                .flex()
                .items_center()
                .gap(px(5.))
                .h(px(TOOLBAR_BUTTON_HEIGHT))
                .px(px(TITLE_PADDING))
                .rounded_sm()
                .text_size(ui_font_size(13.))
                .text_color(rgb(t.fg))
                .cursor_pointer()
                .hover(|style| style.bg(rgb(t.row_alt_bg)))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|view, _: &MouseDownEvent, window, cx| {
                        cx.stop_propagation();
                        view.focus_handle.focus(window, cx);
                        view.open_repo_switcher(window.window_handle(), cx);
                    }),
                )
                .child(picker_opener_bounds(repo_switcher_bounds))
                .child(
                    div()
                        .id("repo-switcher-repository-name")
                        .debug_selector(move || repo_name_selector.clone())
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(SharedString::from(repo_name)),
                )
                .children(repo.workspace.map(|workspace| {
                    let selector = format!("repo-title-workspace-{workspace}");
                    div()
                        .id("repo-switcher-workspace-name")
                        .debug_selector(move || selector.clone())
                        .flex()
                        .items_center()
                        .gap(px(5.))
                        .child(div().text_color(rgb(t.fg_faint)).child("/"))
                        .child(
                            div()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child(workspace),
                        )
                }))
                .child(icons::icon(icons::glyph::CARET_DOWN, 10., t.fg_dim)),
            |view| view.repo_switcher.is_some(),
            RepoWindow::close_repo_switcher,
            cx,
        ))
        .child(div().flex_1())
        .child(revset_bar)
        .child(div().flex_1())
        .child(buttons::tools_cluster(editor, terminal, focused, &t, cx))
        .child(div().flex_none().ml(px(-TOOLBAR_GAP)).child(trailing))
        .into_any_element()
}
