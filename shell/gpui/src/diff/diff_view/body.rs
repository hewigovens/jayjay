use gpui::{
    AnyElement, IntoElement, ParentElement, Styled, UniformListScrollHandle, div, px, rgb,
    uniform_list,
};
use jayjay_core::diff::FileDiff;
use jayjay_core::diff::side_by_side::build_side_by_side_rows;

use crate::app::theme::Theme;
use crate::diff::line::{GUTTER_WIDTH, content_row, gutter_row};
use crate::diff::side_by_side::side_by_side_row;

pub(super) fn unified_body(
    fd: &FileDiff,
    theme: Theme,
    query: Option<String>,
    scroll: UniformListScrollHandle,
) -> AnyElement {
    let lines: std::sync::Arc<Vec<jayjay_core::diff::DiffLine>> =
        std::sync::Arc::new(fd.lines.clone());
    let count = lines.len();
    let theme = std::sync::Arc::new(theme);
    let query = std::sync::Arc::new(query);

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
        move |range: std::ops::Range<usize>, _window, _cx| {
            range
                .map(|ix| content_row(&content_lines[ix], &content_theme, content_query.as_deref()))
                .collect()
        },
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
    let rows: std::sync::Arc<Vec<_>> = std::sync::Arc::new(build_side_by_side_rows(&fd.lines));
    let count = rows.len();
    let theme = std::sync::Arc::new(theme);
    let query = std::sync::Arc::new(query);
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
