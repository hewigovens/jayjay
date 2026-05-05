use std::sync::Arc;

use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ParentElement, Styled, UniformListScrollHandle, div, px, rgb,
    uniform_list,
};
use jayjay_core::diff::FileDiff;
use jayjay_core::diff::side_by_side::build_side_by_side_rows;

use crate::app::theme::Theme;
use crate::diff::line::{GUTTER_WIDTH, content_row, gutter_row};
use crate::diff::side_by_side::side_by_side_row;
use crate::log::LogView;

pub(super) fn unified_body(
    fd: &FileDiff,
    theme: Theme,
    query: Option<String>,
    scroll: UniformListScrollHandle,
    cx: &mut Context<LogView>,
) -> AnyElement {
    let lines: Arc<Vec<jayjay_core::diff::DiffLine>> = Arc::new(fd.lines.clone());
    let count = lines.len();
    let theme = Arc::new(theme);
    let query = Arc::new(query);

    let gutter_lines = lines.clone();
    let gutter_theme = theme.clone();
    let gutter = uniform_list(
        "diff-gutter",
        count,
        move |range: std::ops::Range<usize>, _window, _cx| {
            range
                .map(|ix| gutter_row(&gutter_lines[ix], &gutter_theme))
                .collect()
        },
    )
    .track_scroll(&scroll);

    let content_lines = lines;
    let content_theme = theme.clone();
    let content_query = query;
    let content = uniform_list(
        "diff-content",
        count,
        cx.processor(move |view, range: std::ops::Range<usize>, _window, cx| {
            let sel = view.diff_selection;
            range
                .map(|ix| {
                    let is_selected = sel.is_some_and(|s| s.covers(ix));
                    content_row(
                        &content_lines[ix],
                        &content_theme,
                        content_query.as_deref(),
                        is_selected,
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |v, _: &MouseDownEvent, _, cx| {
                            v.start_diff_selection(ix, cx);
                        }),
                    )
                    .on_mouse_move(cx.listener(move |v, _: &MouseMoveEvent, _, cx| {
                        v.extend_diff_selection(ix, cx);
                    }))
                    .into_any_element()
                })
                .collect()
        }),
    )
    .track_scroll(&scroll);

    div()
        .flex()
        .flex_row()
        .h_full()
        .min_h_0()
        .child(
            div()
                .flex_none()
                .w(px(GUTTER_WIDTH))
                .h_full()
                .border_r_1()
                .border_color(rgb(theme.border))
                .child(gutter),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .h_full()
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|v, _: &MouseUpEvent, _, cx| {
                        v.finish_diff_selection(cx);
                    }),
                )
                .child(crate::ui::primitives::no_scrollbar_gutter(content).h_full()),
        )
        .into_any_element()
}

pub(super) fn side_by_side_body(
    fd: &FileDiff,
    theme: Theme,
    query: Option<String>,
    scroll: UniformListScrollHandle,
) -> AnyElement {
    let rows: Arc<Vec<_>> = Arc::new(build_side_by_side_rows(&fd.lines));
    let count = rows.len();
    let theme = Arc::new(theme);
    let query = Arc::new(query);
    let list = uniform_list(
        "ssv-rows",
        count,
        move |range: std::ops::Range<usize>, _window, _cx| {
            range
                .map(|ix| side_by_side_row(&rows[ix], &theme, query.as_deref()))
                .collect()
        },
    )
    .track_scroll(&scroll);
    crate::ui::primitives::no_scrollbar_gutter(list)
        .h_full()
        .into_any_element()
}
