use gpui::{Div, FontWeight, ParentElement, SharedString, Styled, div, px, rgb};

use crate::app::theme::{FONT_BODY, Theme, ui_font_size};
use crate::ui::icons;

/// A full-width strip for a window-level state; the caller sets the border edge that faces the content.
pub(crate) fn notice_bar(bg: u32, t: &Theme) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(4.))
        .w_full()
        .px(px(12.))
        .py(px(8.))
        .bg(rgb(bg))
        .border_color(rgb(t.border))
        .text_size(ui_font_size(FONT_BODY))
        .text_color(rgb(t.fg))
}

pub(crate) fn notice_bar_header(
    glyph: &'static str,
    accent: u32,
    title: impl Into<SharedString>,
) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(10.))
        .child(icons::icon(glyph, 13., accent))
        .child(
            div()
                .flex_none()
                .font_weight(FontWeight::SEMIBOLD)
                .child(title.into()),
        )
}
