use gpui::{IntoElement, ParentElement, Styled, div, px, rgb};

use crate::app::theme::Theme;
use crate::ui::icons;

pub(super) fn placeholder_card(
    glyph_str: &'static str,
    title: &'static str,
    body: &'static str,
    t: &Theme,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .size_full()
        .items_center()
        .justify_center()
        .gap(px(10.))
        .px(px(40.))
        .bg(rgb(t.detail_bg))
        .child(icons::icon(glyph_str, 28., t.fg_dim))
        .child(div().text_size(px(14.)).text_color(rgb(t.fg)).child(title))
        .child(
            div()
                .text_size(px(11.))
                .text_color(rgb(t.fg_dim))
                .child(body),
        )
}

pub(super) fn placeholder(text: &'static str, t: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_1()
        .size_full()
        .items_center()
        .justify_center()
        .bg(rgb(t.detail_bg))
        .text_color(rgb(t.fg_dim))
        .child(text)
}

pub(super) fn placeholder_inner(text: &'static str, t: &Theme) -> impl IntoElement {
    div()
        .flex()
        .size_full()
        .items_center()
        .justify_center()
        .text_color(rgb(t.fg_dim))
        .child(text)
}
