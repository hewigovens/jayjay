use gpui::{Div, ParentElement, SharedString, Styled, div, px, rgb};

use crate::app::theme::ui_font_size;

/// Avatar followed by the author's name, sized like the surrounding meta text.
pub(crate) fn with_name(email: &str, name: &str, font_size: f32, color: u32) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(5.))
        .child(super::element(email, name, 14.))
        .child(
            div()
                .flex_none()
                .text_size(ui_font_size(font_size))
                .text_color(rgb(color))
                .child(SharedString::from(name.to_owned())),
        )
}
