use gpui::{IntoElement, ParentElement, SharedString, Styled, div, px, rgb};

use crate::app::theme::{FONT_META, Theme};

pub(super) fn file_column_header(
    reviewed: usize,
    count: usize,
    loading: bool,
    show_review: bool,
    t: &Theme,
) -> impl IntoElement {
    let label = if loading {
        String::from("Loading…")
    } else if count == 0 {
        String::from("0 files")
    } else if show_review {
        format!("{reviewed} / {count} reviewed")
    } else {
        format!("{count} files")
    };
    div()
        .px(px(12.))
        .py(px(6.))
        .bg(rgb(t.header_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .text_size(px(FONT_META))
        .text_color(rgb(t.fg_dim))
        .child(SharedString::from(label))
}
