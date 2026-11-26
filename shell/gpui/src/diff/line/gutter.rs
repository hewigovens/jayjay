use gpui::{
    AnyElement, Div, ElementId, InteractiveElement, ParentElement, Pixels, SharedString, Stateful,
    Styled, div, px, rgb, rgba,
};
use jayjay_core::diff::{DiffLine, DiffSpanStyle};
use jayjay_review::ReviewGroupState;

use super::colors::line_bg_color;
use crate::app::fonts::CodeText as _;
use crate::app::theme::{Theme, with_alpha};
use crate::ui::primitives::HelpTag;

pub const REVIEW_STRIPE_WIDTH: f32 = 6.;
const NUMBER_PADDING: f32 = 7.;

/// Sized to the file's largest line number in the code font, like the SwiftUI gutter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GutterWidths {
    pub number: f32,
    pub dot: f32,
}

impl GutterWidths {
    /// Line numbers only grow down a diff, so the last numbered lines hold the largest.
    pub fn largest_line_number(lines: &[DiffLine]) -> u32 {
        let last = |number: fn(&DiffLine) -> Option<u32>| lines.iter().rev().find_map(number);
        last(|line| line.old_line_no)
            .max(last(|line| line.new_line_no))
            .unwrap_or(0)
    }

    pub fn new(largest_line_number: u32, advance: Pixels) -> Self {
        let digits = largest_line_number.max(10).ilog10() + 1;
        let advance = f32::from(advance);
        Self {
            number: (digits as f32 * advance + NUMBER_PADDING).ceil(),
            dot: (advance * 2.).ceil(),
        }
    }

    pub fn interactive(&self, shows_review: bool) -> f32 {
        self.number * 2.
            + self.dot
            + if shows_review {
                REVIEW_STRIPE_WIDTH
            } else {
                0.
            }
    }

    // `bg` must exactly match the caller's computed line background — a mismatch paints an opaque fill over the added/removed tint.
    pub fn number_cell(&self, text: String, theme: &Theme, bg: u32) -> Div {
        div()
            .flex()
            .items_center()
            .justify_end()
            .flex_none()
            .w(px(self.number))
            .h(px(theme.code_line_height()))
            .pl(px(2.))
            .pr(px(5.))
            .text_color(rgb(theme.diff_gutter_fg))
            .bg(rgb(bg))
            .line_height(px(theme.code_line_height()))
            .child(SharedString::from(text))
    }
}

pub fn gutter_column(theme: &Theme, widths: GutterWidths, shows_review: bool) -> Div {
    div()
        .flex_none()
        .w(px(widths.interactive(shows_review)))
        .h_full()
        .bg(rgb(theme.diff_gutter_bg))
        .border_r_1()
        .border_color(rgb(theme.border))
}

/// Continuation fragments and separators take a blank dot cell; only the fragment carrying the line numbers renders a dot.
pub fn interactive_gutter_row(
    line: &DiffLine,
    theme: &Theme,
    widths: GutterWidths,
    is_selected: bool,
    review_cell: Option<AnyElement>,
    dot_cell: AnyElement,
) -> Div {
    let width = widths.interactive(review_cell.is_some());
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
        .code_text(12., theme)
        .line_height(px(theme.code_line_height()));
    if let Some(cell) = review_cell {
        row = row.child(cell);
    }
    row = row
        .child(widths.number_cell(old_no, theme, bg))
        .child(widths.number_cell(new_no, theme, bg))
        .child(dot_cell)
        .child(hover_overlay(theme));
    if is_selected {
        row = row.child(selection_stripe(theme));
    }
    row
}

pub fn review_stripe(
    id: impl Into<ElementId>,
    state: ReviewGroupState,
    theme: &Theme,
) -> Stateful<Div> {
    let (color, label) = match state {
        ReviewGroupState::Reviewed => (theme.selected_accent, jayjay_i18n::repo::reviewed()),
        ReviewGroupState::Unreviewed => (theme.selected_bg, jayjay_i18n::repo::unreviewed()),
        ReviewGroupState::ChangedSinceReview => {
            (theme.note_accent, jayjay_i18n::repo::changed_since_review())
        }
    };
    div()
        .id(id)
        .flex_none()
        .w(px(REVIEW_STRIPE_WIDTH))
        .h_full()
        .bg(rgb(color))
        .cursor_pointer()
        .hover(move |s| s.bg(rgba(with_alpha(color, 0xb8))))
        .help(label)
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
