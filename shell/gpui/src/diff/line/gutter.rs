//! Layout for the diff gutter: line-number cells, note-dot column, and selection highlight.

use gpui::{AnyElement, Div, InteractiveElement, ParentElement, Styled, div, px, rgb, rgba};
use jayjay_core::diff::{DiffLine, DiffSpanStyle};

use super::{GUTTER_NUMBER_WIDTH, gutter_cell, line_bg_color};
use crate::app::fonts;
use crate::app::theme::{Theme, ui_font_size, with_alpha};

pub const NOTE_DOT_WIDTH: f32 = 14.;
pub const INTERACTIVE_GUTTER_WIDTH: f32 = GUTTER_NUMBER_WIDTH * 2. + NOTE_DOT_WIDTH;

pub fn interactive_gutter_column(theme: &Theme) -> Div {
    div()
        .flex_none()
        .w(px(INTERACTIVE_GUTTER_WIDTH))
        .h_full()
        .bg(rgb(theme.diff_gutter_bg))
        .border_r_1()
        .border_color(rgb(theme.border))
}

/// `dot_cell` must be a blank `note_dot_cell(None, theme)` for continuation fragments and separators — only the fragment carrying the line numbers actually renders a dot.
pub fn interactive_gutter_row(
    line: &DiffLine,
    theme: &Theme,
    is_selected: bool,
    dot_cell: AnyElement,
) -> Div {
    if line.style == DiffSpanStyle::Separator {
        let mut row = div()
            .relative()
            .w(px(INTERACTIVE_GUTTER_WIDTH))
            .h(px(theme.code_line_height()))
            .bg(rgb(theme.diff_separator_bg));
        if is_selected {
            row = row.child(selection_stripe(theme));
        }
        return row;
    }

    let bg = line_bg_color(line.style, line.conflict_kind, theme);
    let old_no = line.old_line_no.map(|n| n.to_string()).unwrap_or_default();
    let new_no = line.new_line_no.map(|n| n.to_string()).unwrap_or_default();

    let mut row = div()
        .relative()
        .flex()
        .flex_row()
        .w(px(INTERACTIVE_GUTTER_WIDTH))
        .h(px(theme.code_line_height()))
        .bg(rgb(bg))
        .font_family(fonts::mono())
        .text_size(ui_font_size(12.))
        .line_height(px(theme.code_line_height()))
        .child(gutter_cell(old_no, theme, bg))
        .child(gutter_cell(new_no, theme, bg))
        .child(dot_cell)
        .child(hover_overlay(theme));
    if is_selected {
        row = row.child(selection_stripe(theme));
    }
    row
}

/// MUST be the row's last child, like `selection_stripe` — siblings paint in declaration order.
pub fn content_row_tint(theme: &Theme) -> Div {
    row_overlay(theme, 0x22)
}

// MUST be the row's last child — siblings paint in declaration order.
fn selection_stripe(theme: &Theme) -> Div {
    row_overlay(theme, 0x55)
}

/// Must be added after the number/dot cells and before an optional `selection_stripe` — their opaque fills would otherwise hide a tint painted before them.
fn hover_overlay(theme: &Theme) -> Div {
    div()
        .absolute()
        .left(px(0.))
        .top(px(0.))
        .w_full()
        .h(px(theme.code_line_height()))
        .hover(|s| s.bg(rgba(with_alpha(theme.fg, 0x0e))))
}

fn row_overlay(theme: &Theme, alpha: u8) -> Div {
    div()
        .absolute()
        .left(px(0.))
        .top(px(0.))
        .w_full()
        .h(px(theme.code_line_height()))
        .bg(rgba(with_alpha(theme.selected_bg, alpha)))
}
