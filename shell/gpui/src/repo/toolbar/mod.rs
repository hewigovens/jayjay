mod buttons;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, SharedString, Styled, div, px, rgb,
};

use crate::app::theme::theme;
use crate::app::tools;
use crate::platform::TOOLBAR_LEADING_INSET;
use crate::repo::window::RepoWindow;
use crate::ui::icons;
use crate::ui::primitives::TOOLBAR_BUTTON_HEIGHT;

const TOOLBAR_HEIGHT: f32 = 44.;

pub(crate) struct ToolbarActivity {
    pub(crate) has_wc_changes: bool,
    pub(crate) is_refreshing: bool,
    pub(crate) is_fetching: bool,
    pub(crate) is_pushing: bool,
}

pub(crate) fn toolbar(
    repo_path: SharedString,
    bookmark_count: usize,
    revset_filter_visible: bool,
    activity: ToolbarActivity,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let t = theme(cx).clone();

    let repo_name = repo_path
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(repo_path.as_ref())
        .to_owned();
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
            cx.listener(|_, ev: &MouseDownEvent, window, _cx| {
                if ev.click_count == 2 {
                    window.zoom_window();
                }
            }),
        )
        .child(buttons::bookmarks_button(bookmark_count, &t, cx))
        .child(buttons::divider(&t))
        .child(buttons::sync_cluster(
            revset_filter_visible,
            activity,
            &t,
            cx,
        ))
        .child(
            div()
                .id("repo-switcher-button")
                .debug_selector(|| "repo-switcher-button".to_owned())
                .flex()
                .items_center()
                .gap(px(5.))
                .h(px(TOOLBAR_BUTTON_HEIGHT))
                .px(px(10.))
                .rounded_sm()
                .text_size(px(13.))
                .text_color(rgb(t.fg))
                .cursor_pointer()
                .hover(|style| style.bg(rgb(t.row_alt_bg)))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|view, ev: &MouseDownEvent, window, cx| {
                        cx.stop_propagation();
                        view.open_repo_switcher(ev.position, window.window_handle(), cx);
                    }),
                )
                .child(SharedString::from(repo_name))
                .child(icons::icon(icons::glyph::CARET_DOWN, 10., t.fg_dim)),
        )
        .child(div().flex_1())
        .child(buttons::tools_cluster(
            repo_path,
            open_editor_label,
            open_terminal_label,
            &t,
            cx,
        ))
        .into_any_element()
}
