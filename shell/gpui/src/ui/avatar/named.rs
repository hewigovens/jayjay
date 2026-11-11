use gpui::{Div, ParentElement, SharedString, Styled, div, px, rgb};
use jayjay_core::CommitAuthor;

use crate::app::theme::ui_font_size;

/// Avatar followed by the author's name, sized like the surrounding meta text.
pub(crate) fn with_name(author: &CommitAuthor, font_size: f32, color: u32) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(5.))
        .child(super::element(author, 14.))
        .child(
            div()
                .flex_none()
                .text_size(ui_font_size(font_size))
                .text_color(rgb(color))
                .child(SharedString::from(author.name.clone())),
        )
}
