use gpui::{Styled, rgb, transparent_black};

use crate::app::theme::Theme;

/// Accent ring on the Tab-focused control; the transparent border keeps the unfocused layout identical.
pub(crate) fn focus_ring<E: Styled>(element: E, focused: bool, t: &Theme) -> E {
    element.border_1().border_color(if focused {
        rgb(t.selected_accent).into()
    } else {
        transparent_black()
    })
}
