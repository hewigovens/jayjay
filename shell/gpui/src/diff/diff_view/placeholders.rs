use gpui::{IntoElement, ParentElement, SharedString, Styled, div, px, rgb};

use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons;

pub(super) fn placeholder_card(
    glyph_str: &'static str,
    title: impl Into<SharedString>,
    body: impl Into<SharedString>,
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
                .child(title.into()),
        )
        .child(
            div()
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.fg_dim))
                .child(body.into()),
        )
}

pub(super) fn placeholder(
    title: impl Into<SharedString>,
    body: impl Into<SharedString>,
    t: &Theme,
) -> impl IntoElement {
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
                .child(title.into()),
        )
        .child(
            div()
                .text_size(ui_font_size(12.))
                .text_color(rgb(t.fg_faint))
                .child(body.into()),
        )
}

pub(super) fn placeholder_inner(text: impl Into<SharedString>, t: &Theme) -> impl IntoElement {
    div()
        .flex()
        .size_full()
        .items_center()
        .justify_center()
        .text_color(rgb(t.fg_dim))
        .child(text.into())
}
