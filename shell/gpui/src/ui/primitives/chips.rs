use gpui::{Div, ParentElement, SharedString, Styled, div, px, rgb};

use crate::app::theme::ui_font_size;
use crate::ui::icons;

pub(crate) fn capsule(label: impl Into<SharedString>, bg: u32, fg: u32, font_size: f32) -> Div {
    div()
        .flex_none()
        .px(px(6.))
        .py(px(1.))
        .rounded_full()
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .text_size(ui_font_size(font_size))
        .child(label.into())
}

/// Returns `Div`, not `impl IntoElement`, so callers can chain `.id()` / `.on_mouse_down()`.
pub(crate) fn icon_chip(
    glyph_str: &'static str,
    label: impl Into<SharedString>,
    bg: u32,
    fg: u32,
    icon_color: u32,
    font_size: f32,
) -> Div {
    div()
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(3.))
        .px(px(6.))
        .py(px(1.))
        .rounded_full()
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .text_size(ui_font_size(font_size))
        .child(icons::icon(glyph_str, font_size, icon_color))
        .child(label.into())
}

pub(crate) fn icon_label(
    glyph_str: &'static str,
    label: impl Into<SharedString>,
    icon_size: f32,
    icon_color: u32,
) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .child(icons::icon(glyph_str, icon_size, icon_color))
        .child(label.into())
}
