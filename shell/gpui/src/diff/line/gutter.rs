//! Layout for the diff gutter: line-number cells, note-dot column, and selection highlight.

use gpui::{
    AnyElement, Div, ElementId, InteractiveElement, ParentElement, Stateful,
    StatefulInteractiveElement, Styled, div, px, rgb, rgba,
};
use jayjay_core::diff::{DiffLine, DiffSpanStyle};
use jayjay_review::ReviewGroupState;

use super::{GUTTER_NUMBER_WIDTH, gutter_cell, line_bg_color};
use crate::app::fonts;
use crate::app::theme::{Theme, ui_font_size, with_alpha};
use crate::ui::primitives::text_tooltip;

pub const NOTE_DOT_WIDTH: f32 = 14.;
const INTERACTIVE_GUTTER_WIDTH: f32 = GUTTER_NUMBER_WIDTH * 2. + NOTE_DOT_WIDTH;
pub const REVIEW_STRIPE_WIDTH: f32 = 6.;

pub fn interactive_gutter_column(theme: &Theme, shows_review: bool) -> Div {
    div()
        .flex_none()
        .w(px(interactive_gutter_width(shows_review)))
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
    review_cell: Option<AnyElement>,
    dot_cell: AnyElement,
) -> Div {
    let width = interactive_gutter_width(review_cell.is_some());
    if line.style == DiffSpanStyle::Separator {
        let mut row = div()
            .relative()
            .flex()
            .flex_row()
            .w(px(width))
            .h(px(theme.code_line_height()))
            .bg(rgb(theme.diff_separator_bg));
        if let Some(cell) = review_cell {
            row = row.child(cell);
        }
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
        .w(px(width))
        .h(px(theme.code_line_height()))
        .bg(rgb(bg))
        .font_family(fonts::mono())
        .text_size(ui_font_size(12.))
        .line_height(px(theme.code_line_height()));
    if let Some(cell) = review_cell {
        row = row.child(cell);
    }
    row = row
        .child(gutter_cell(old_no, theme, bg))
        .child(gutter_cell(new_no, theme, bg))
        .child(dot_cell)
        .child(hover_overlay(theme));
    if is_selected {
        row = row.child(selection_stripe(theme));
    }
    row
}

pub(super) fn interactive_gutter_width(shows_review: bool) -> f32 {
    INTERACTIVE_GUTTER_WIDTH
        + if shows_review {
            REVIEW_STRIPE_WIDTH
        } else {
            0.
        }
}

pub fn review_stripe(
    id: impl Into<ElementId>,
    state: ReviewGroupState,
    theme: &Theme,
) -> Stateful<Div> {
    let (color, label) = match state {
        ReviewGroupState::Reviewed => (theme.file_added_color, "Reviewed"),
        ReviewGroupState::Unreviewed => (theme.selected_bg, "Unreviewed"),
        ReviewGroupState::ChangedSinceReview => (theme.file_modified_color, "Changed since review"),
    };
    div()
        .id(id)
        .flex_none()
        .w(px(REVIEW_STRIPE_WIDTH))
        .h_full()
        .bg(rgb(color))
        .cursor_pointer()
        .hover(move |s| s.bg(rgba(with_alpha(color, 0xb8))))
        .tooltip(text_tooltip(label))
}

pub fn review_stripe_spacer() -> Div {
    div().flex_none().w(px(REVIEW_STRIPE_WIDTH)).h_full()
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
