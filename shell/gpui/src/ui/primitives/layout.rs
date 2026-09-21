use gpui::{Div, IntoElement, ParentElement, Styled, UniformList, div, px, rgb};

use crate::app::theme::Theme;

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
