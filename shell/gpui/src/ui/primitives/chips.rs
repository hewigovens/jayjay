use gpui::{Div, ParentElement, SharedString, Styled, div, rgb};

use crate::app::fonts::CodeText as _;
use crate::app::theme::{Theme, ui_size};
use crate::ui::icons;

pub(crate) fn capsule(label: impl Into<SharedString>, bg: u32, fg: u32, font_size: f32) -> Div {
    div()
        .flex_none()
        .px(ui_size(6.))
        .py(ui_size(1.))
        .rounded_full()
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .text_size(ui_size(font_size))
        .child(label.into())
}

pub(crate) fn key_cap(label: impl Into<SharedString>, t: &Theme) -> Div {
    div()
        .flex_none()
        .min_w(ui_size(18.))
        .px(ui_size(6.))
        .py(ui_size(2.))
        .rounded(ui_size(5.))
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(t.row_alt_bg))
        .code_font()
        .text_size(ui_size(12.))
        .text_color(rgb(t.fg))
        .text_center()
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
        .gap(ui_size(3.))
        .px(ui_size(6.))
        .py(ui_size(1.))
        .rounded_full()
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .text_size(ui_size(font_size))
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
        .gap(ui_size(6.))
        .child(icons::icon(glyph_str, icon_size, icon_color))
        .child(label.into())
}
