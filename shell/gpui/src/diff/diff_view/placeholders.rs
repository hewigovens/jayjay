use gpui::{IntoElement, ParentElement, Styled, div, px, rgb};

use crate::app::theme::{Theme, ui_font_size};
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
        .child(
            div()
                .text_size(ui_font_size(14.))
                .text_color(rgb(t.fg))
                .child(title),
        )
        .child(
            div()
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.fg_dim))
                .child(body),
        )
}

pub(super) fn placeholder(title: &'static str, body: &'static str, t: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_1()
        .flex_col()
        .size_full()
        .items_center()
        .justify_center()
        .gap(px(6.))
        .bg(rgb(t.detail_bg))
        .child(
            div()
                .text_size(ui_font_size(17.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(t.fg_dim))
                .child(title),
        )
        .child(
            div()
                .text_size(ui_font_size(12.))
                .text_color(rgb(t.fg_faint))
                .child(body),
        )
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
