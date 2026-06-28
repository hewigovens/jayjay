mod buttons;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, SharedString, Styled, div, px, rgb,
};

use crate::app::theme::theme;
use crate::app::tools;
use crate::platform::TOOLBAR_LEADING_INSET;
use crate::repo::window::RepoWindow;
use crate::ui::icons::glyph;
use buttons::{RepoToolAction, SyncAction};

const TOOLBAR_HEIGHT: f32 = 44.;

pub fn toolbar(
    repo_path: SharedString,
    bookmark_count: usize,
    has_wc_changes: bool,
    is_refreshing: bool,
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
        .bg(rgb(t.header_bg))
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
        .child(buttons::refresh_button(
            has_wc_changes,
            is_refreshing,
            &t,
            cx,
        ))
        .child(buttons::sync_button(
            glyph::ARROW_DOWN,
            "tb-pull",
            "Pull",
            SyncAction::FetchOrigin,
            &t,
            cx,
        ))
        .child(buttons::sync_button(
            glyph::ARROW_UP,
            "tb-push",
            "Push",
            SyncAction::PushDefault,
            &t,
            cx,
        ))
        .child(div().flex_1())
        .child(
            div()
                .text_size(px(13.))
                .text_color(rgb(t.fg))
                .child(SharedString::from(repo_name)),
        )
        .child(div().flex_1())
        .child(buttons::repo_tool_button(
            "tb-open-editor",
            glyph::FILE_CODE,
            repo_path.clone(),
            RepoToolAction::Editor,
            open_editor_label,
            &t,
            cx,
        ))
        .child(buttons::repo_tool_button(
            "tb-open-terminal",
            glyph::TERMINAL,
            repo_path,
            RepoToolAction::Terminal,
            open_terminal_label,
            &t,
            cx,
        ))
        .child(buttons::settings_button(&t))
        .into_any_element()
}
