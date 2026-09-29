use std::ops::Range;

use gpui::{Div, FontWeight, ParentElement, Pixels, SharedString, Styled, div, px, rgb, rgba};
use jayjay_core::diff::{ConflictLineKind, DiffLine, DiffSpanStyle, conflict_display_text};

use crate::app::fonts;
use crate::app::theme::{Theme, ui_font_size};

use super::super::spans::span_element;
use super::colors::{line_bg_color, line_text_color};

pub fn content_row(
    line: &DiffLine,
    theme: &Theme,
    find_query: Option<&str>,
    selection_cols: Option<Range<usize>>,
    advance: Pixels,
) -> Div {
    if line.style == DiffSpanStyle::Separator {
        return separator_content(line, theme, selection_cols.is_some());
    }
    let bg = line_bg_color(line.style, line.conflict_kind, theme);
    let base_text_fg = line_text_color(line.style, line.conflict_kind, theme);

    if let Some(label) = conflict_label(line) {
        let mut row = div()
            .relative()
            .flex()
            .items_center()
            .w_full()
            .h(px(theme.code_line_height()))
            .bg(rgb(bg))
            .font_family(fonts::mono())
            .text_size(ui_font_size(12.))
            .line_height(px(theme.code_line_height()))
            .text_color(rgb(base_text_fg))
            .font_weight(FontWeight::MEDIUM)
            .px(px(16.))
            .child(conflict_stripe_overlay(line.conflict_kind, theme))
            .child(SharedString::from(conflict_display_line(
                label,
                line.conflict_kind,
            )));
        if let Some(cols) = selection_cols {
            row = row.child(selection_overlay(cols, advance, theme));
        }
        return row;
    }

    let mut text_row = div()
        .flex()
        .flex_row()
        .flex_1()
        .min_w_0()
        .h(px(theme.code_line_height()));
    for span in &line.spans {
        text_row = text_row.child(span_element(
            span,
            base_text_fg,
            line.style,
            theme,
            find_query,
        ));
    }

    let mut row = div()
        .relative()
        .flex()
        .flex_row()
        .w_full()
        .h(px(theme.code_line_height()))
        .bg(rgb(bg))
        .font_family(fonts::mono())
        .text_size(ui_font_size(12.))
        .line_height(px(theme.code_line_height()))
        .child(conflict_stripe_overlay(line.conflict_kind, theme))
        .child(text_row);
    if let Some(cols) = selection_cols {
        row = row.child(selection_overlay(cols, advance, theme));
    }
    row
}

pub fn conflict_stripe_overlay(kind: ConflictLineKind, theme: &Theme) -> Div {
    let color = if kind == ConflictLineKind::None {
        0x000000
    } else {
        theme.diff_conflict_stripe
    };
    let opacity = if kind == ConflictLineKind::None {
        0.
    } else {
        1.
    };
    div()
        .absolute()
        .left(px(0.))
        .top(px(0.))
        .w(px(3.))
        .h_full()
        .bg(rgba((color << 8) | ((opacity * 255.) as u32)))
}

// MUST be the row's last child — siblings paint in declaration order.
pub fn selection_overlay(cols: Range<usize>, advance: Pixels, theme: &Theme) -> Div {
    let left = cols.start as f32 * f32::from(advance);
    let width = (cols.end.saturating_sub(cols.start)) as f32 * f32::from(advance);
    let bg = rgba(((theme.selected_bg as u64) << 8) as u32 | 0x66);
    div()
        .absolute()
        .left(px(left))
        .top(px(0.))
        .w(px(width.max(2.)))
        .h(px(theme.code_line_height()))
        .bg(bg)
}

pub(super) fn conflict_label(line: &DiffLine) -> Option<String> {
    match line.conflict_kind {
        ConflictLineKind::Start | ConflictLineKind::End | ConflictLineKind::Section => {
            conflict_display_text(line.conflict_kind, &line.text())
        }
        _ => None,
    }
}

fn conflict_display_line(label: String, kind: ConflictLineKind) -> String {
    match kind {
        ConflictLineKind::Section => format!("    {label}"),
        _ => format!("  {label}"),
    }
}

fn separator_content(line: &DiffLine, theme: &Theme, is_selected: bool) -> Div {
    let label = line.text();
    let label = if label.is_empty() {
        String::from("…")
    } else {
        label
    };
    let bg = if is_selected {
        theme.selected_bg
    } else {
        theme.diff_separator_bg
    };
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(theme.code_line_height()))
        .bg(rgb(bg))
        .font_family(fonts::mono())
        .text_size(px(theme.compact_code_font_size()))
        .line_height(px(theme.code_line_height()))
        .text_color(rgb(theme.diff_text_dim))
        .px(px(20.))
        .child(SharedString::from(label))
}
