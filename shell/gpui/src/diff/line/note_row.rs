//! Renders review-note rows: a gutter dot cell and a content bubble row, both sized to the diff line height to fit the `uniform_list`.

use gpui::{Div, ParentElement, Pixels, SharedString, Styled, div, px, rgb, rgba};

use crate::app::fonts;
use crate::app::theme::{Theme, ui_font_size, with_alpha};
use crate::diff::NoteDotKind;

use super::NOTE_DOT_WIDTH;
use super::gutter::interactive_gutter_width;

fn note_accent(theme: &Theme) -> u32 {
    theme.file_modified_color
}

/// `bg` must match the anchor line's gutter background (or the neutral gutter bg when blank) — a mismatch shows as a color seam against the number cells.
pub fn note_dot_cell(dot: Option<NoteDotKind>, theme: &Theme, bg: u32) -> Div {
    let (glyph, color): (&str, u32) = match dot {
        Some(NoteDotKind::Active) => ("●", note_accent(theme)),
        Some(NoteDotKind::Resolved) => ("●", theme.fg_faint),
        None => ("", theme.diff_gutter_fg),
    };
    div()
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .w(px(NOTE_DOT_WIDTH))
        .h(px(theme.code_line_height()))
        .bg(rgb(bg))
        .text_size(ui_font_size(9.))
        .text_color(rgb(color))
        .line_height(px(theme.code_line_height()))
        .child(SharedString::from(glyph))
}

/// Blank gutter row under a note-bubble content row — notes have no line number of their own.
pub fn note_gutter_row(theme: &Theme, shows_review: bool) -> Div {
    div()
        .w(px(interactive_gutter_width(shows_review)))
        .h(px(theme.code_line_height()))
        .bg(rgb(theme.diff_gutter_bg))
}

/// Only the first/last fragment rounds its outer corners, so a multi-row bubble reads as one continuous card; `indent` shifts the whole card to the anchored line's leading-whitespace column.
pub fn note_content_row(
    text: SharedString,
    theme: &Theme,
    is_first: bool,
    is_last: bool,
    indent: Pixels,
) -> Div {
    let accent = note_accent(theme);
    let fill_alpha = if theme.is_dark { 0x21 } else { 0x17 };
    let mut bubble = div()
        .flex()
        .items_center()
        .flex_1()
        .min_w_0()
        .h(px(theme.code_line_height()))
        .px(px(16.))
        .bg(rgba(with_alpha(accent, fill_alpha)))
        .border_l_1()
        .border_r_1()
        .border_color(rgba(with_alpha(accent, 0x59)))
        .font_family(fonts::mono())
        .text_size(ui_font_size(12.))
        .text_color(rgb(theme.fg))
        .line_height(px(theme.code_line_height()))
        .child(text);
    if is_first {
        bubble = bubble.border_t_1().rounded_t_md();
    }
    if is_last {
        bubble = bubble.border_b_1().rounded_b_md();
    }
    div()
        .flex()
        .flex_row()
        .w_full()
        .h(px(theme.code_line_height()))
        .child(div().flex_none().w(indent).h(px(theme.code_line_height())))
        .child(bubble)
}
