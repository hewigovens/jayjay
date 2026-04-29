use gpui::{AnyElement, IntoElement, Styled, UniformListScrollHandle, uniform_list};
use jayjay_core::diff::FileDiff;
use jayjay_core::diff::side_by_side::build_side_by_side_rows;

use crate::app::theme::Theme;
use crate::diff::line::diff_line;
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
    let list = uniform_list(
        "diff-lines",
        count,
        move |range: std::ops::Range<usize>, _window, _cx| {
            range
                .map(|ix| diff_line(&lines[ix], &theme, query.as_deref()))
                .collect()
        },
    )
    .track_scroll(&scroll);
    crate::ui::primitives::no_scrollbar_gutter(list)
        .h_full()
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
