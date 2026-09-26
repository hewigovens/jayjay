use gpui::{
    AnyElement, Div, IntoElement, ParentElement, SharedString, Styled, UniformList, div, px, rgb,
};

use crate::app::theme::{Theme, ui_font_size};

/// uniform_list reserves a 15px gutter for an OS scrollbar by default; we don't render one, so collapse it to 0.
pub(crate) fn no_scrollbar_gutter(mut list: UniformList) -> UniformList {
    list.style().scrollbar_width = Some(px(0.).into());
    list
}

pub(crate) fn divider_h(theme: &Theme) -> impl IntoElement {
    div().h(px(1.)).w_full().bg(rgb(theme.border))
}

pub(crate) fn divider_v(theme: &Theme) -> impl IntoElement {
    div().w(px(1.)).h_full().bg(rgb(theme.border))
}

pub(crate) fn dot_separator(theme: &Theme) -> Div {
    div().flex_none().text_color(rgb(theme.fg_faint)).child("·")
}

/// Centered status text filling a window body: loading, empty, or unavailable.
pub(crate) fn placeholder(text: impl Into<SharedString>, theme: &Theme) -> AnyElement {
    centered_message(text.into(), theme.fg_dim)
}

pub(crate) fn placeholder_err(text: &SharedString, theme: &Theme) -> AnyElement {
    centered_message(text.clone(), theme.error_fg)
}

fn centered_message(text: SharedString, color: u32) -> AnyElement {
    div()
        .flex()
        .flex_1()
        .items_center()
        .justify_center()
        .px(px(24.))
        .text_size(ui_font_size(12.))
        .text_color(rgb(color))
        .child(text)
        .into_any_element()
}
