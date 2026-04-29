use gpui::{IntoElement, ParentElement, SharedString, Styled, UniformList, div, px, rgb};

use crate::app::theme::Theme;

/// uniform_list reserves a 15px gutter for an OS scrollbar by default. We
/// don't render a scrollbar, so the gutter just leaves a thick gap on the
/// right edge — collapse it to 0.
pub fn no_scrollbar_gutter(mut list: UniformList) -> UniformList {
    list.style().scrollbar_width = Some(px(0.).into());
    list
}

pub fn capsule(
    label: impl Into<SharedString>,
    bg: u32,
    fg: u32,
    font_size: f32,
) -> impl IntoElement {
    div()
        .flex_none()
        .px(px(6.))
        .py(px(1.))
        .rounded_full()
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .text_size(px(font_size))
        .child(label.into())
}

pub fn divider_h(theme: &Theme) -> impl IntoElement {
    div().h(px(1.)).w_full().bg(rgb(theme.border))
}

pub fn divider_v(theme: &Theme) -> impl IntoElement {
    div().w(px(1.)).h_full().bg(rgb(theme.border))
}
