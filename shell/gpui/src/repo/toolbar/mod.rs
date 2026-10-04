mod buttons;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, SharedString, Styled, div, px, rgb,
};

use crate::app::theme::{theme, ui_font_size};
use crate::app::{repositories, tools};
use crate::platform::{TOOLBAR_HEIGHT, TOOLBAR_LEADING_INSET};
use crate::repo::view_model::RefreshMode;
use crate::repo::window::{FocusStop, RepoWindow, picker_opener};
use crate::ui::icons;
use crate::ui::primitives::TOOLBAR_BUTTON_HEIGHT;

pub(crate) struct ToolbarActivity {
    pub(crate) is_refreshing: bool,
    pub(crate) refresh: RefreshMode,
    pub(crate) is_fetching: bool,
    pub(crate) is_pushing: bool,
}

pub(crate) struct ToolbarRepo {
    pub(crate) path: SharedString,
    pub(crate) root_path: SharedString,
    /// Shown after the repository name only when the repo has more than one workspace.
    pub(crate) workspace: Option<SharedString>,
}

pub(crate) fn toolbar(
    repo: ToolbarRepo,
    sidebar_hidden: bool,
    revset_bar: AnyElement,
    activity: ToolbarActivity,
    focused: Option<FocusStop>,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let t = theme(cx).clone();

    let repo_name = repositories::repository_name(&repo.root_path);
    let repo_name_selector = format!("repo-title-repository-{repo_name}");
    let open_editor_label = SharedString::from(tools::open_in_editor_label(cx));
    let open_terminal_label = SharedString::from(tools::open_in_terminal_label(cx));

    div()
        .id(SharedString::from("toolbar"))
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(TOOLBAR_HEIGHT))
        .pl(px(TOOLBAR_LEADING_INSET))
        .pr(px(12.))
        .gap(px(6.))
        .bg(rgb(t.toolbar_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|view, ev: &MouseDownEvent, window, cx| {
                if ev.click_count == 2 {
                    window.zoom_window();
                }
                // Title-bar clicks take no focus of their own, so an open revset edit would otherwise survive them.
                let on_bar = view
                    .revset_bar_bounds
                    .get()
                    .is_some_and(|bounds| bounds.contains(&ev.position));
                if view.revset_editor.is_some() && !on_bar {
                    view.focus_handle.focus(window, cx);
                }
            }),
        )
        .child(buttons::sync_cluster(
            sidebar_hidden,
            activity,
            focused,
            &t,
            cx,
        ))
        .child(picker_opener(
            div()
                .id("repo-switcher-button")
                .debug_selector(|| "repo-switcher-button".to_owned())
                .flex()
                .items_center()
                .gap(px(5.))
                .h(px(TOOLBAR_BUTTON_HEIGHT))
                .px(px(10.))
                .rounded_sm()
                .text_size(ui_font_size(13.))
                .text_color(rgb(t.fg))
                .cursor_pointer()
                .hover(|style| style.bg(rgb(t.row_alt_bg)))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|view, ev: &MouseDownEvent, window, cx| {
                        cx.stop_propagation();
                        view.focus_handle.focus(window, cx);
                        view.open_repo_switcher(ev.position, window.window_handle(), cx);
                    }),
                )
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
        .child(buttons::tools_cluster(
            open_editor_label,
            open_terminal_label,
            focused,
            &t,
            cx,
        ))
        .into_any_element()
}
