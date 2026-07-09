use std::ops::Range;

use gpui::{
    AnyElement, Div, FontWeight, IntoElement, ParentElement, Pixels, SharedString, Styled, div, px,
    rgb, rgba,
};
use jayjay_core::diff::{ConflictLineKind, DiffLine, DiffSpanStyle, conflict_display_text};
use jayjay_core::{DiffHunk, HunkType};

use crate::app::fonts;
use crate::app::theme::Theme;

use super::spans::span_element;

pub const ROW_HEIGHT: f32 = 18.;
pub const GUTTER_NUMBER_WIDTH: f32 = 34.;
pub const GUTTER_WIDTH: f32 = GUTTER_NUMBER_WIDTH * 2.;

pub fn gutter_column(theme: &Theme) -> Div {
    div()
        .flex_none()
        .w(px(GUTTER_WIDTH))
        .h_full()
        .bg(rgb(theme.diff_gutter_bg))
        .border_r_1()
        .border_color(rgb(theme.border))
}

pub fn gutter_row(line: &DiffLine, theme: &Theme) -> AnyElement {
    if line.style == DiffSpanStyle::Separator {
        return separator_gutter(theme);
    }
    let bg = line_bg_color(line.style, line.conflict_kind, theme);
    let old_no = line.old_line_no.map(|n| n.to_string()).unwrap_or_default();
    let new_no = line.new_line_no.map(|n| n.to_string()).unwrap_or_default();

    div()
        .flex()
        .flex_row()
        .w(px(GUTTER_WIDTH))
        .h(px(ROW_HEIGHT))
        .bg(rgb(bg))
        .font_family(fonts::mono())
        .text_size(px(12.))
        .line_height(px(ROW_HEIGHT))
        .child(gutter_cell(old_no, theme))
        .child(gutter_cell(new_no, theme))
        .into_any_element()
}

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
            .h(px(ROW_HEIGHT))
            .bg(rgb(bg))
            .font_family(fonts::mono())
            .text_size(px(12.))
            .line_height(px(ROW_HEIGHT))
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

    let mut text_row = div().flex().flex_row().flex_1().min_w_0().h(px(ROW_HEIGHT));
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
        .h(px(ROW_HEIGHT))
        .bg(rgb(bg))
        .font_family(fonts::mono())
        .text_size(px(12.))
        .line_height(px(ROW_HEIGHT))
        .child(conflict_stripe_overlay(line.conflict_kind, theme))
        .child(text_row);
    if let Some(cols) = selection_cols {
        row = row.child(selection_overlay(cols, advance, theme));
    }
    row
}

pub fn line_bg_color(style: DiffSpanStyle, conflict_kind: ConflictLineKind, theme: &Theme) -> u32 {
    match conflict_kind {
        ConflictLineKind::Start => theme.diff_conflict_header_bg,
        ConflictLineKind::End | ConflictLineKind::Section => theme.diff_conflict_section_bg,
        ConflictLineKind::Content => theme.diff_conflict_content_bg,
        ConflictLineKind::Added => theme.diff_added_bg,
        ConflictLineKind::Removed => theme.diff_removed_bg,
        ConflictLineKind::None => match style {
            DiffSpanStyle::Added => theme.diff_added_bg,
            DiffSpanStyle::Removed => theme.diff_removed_bg,
            DiffSpanStyle::Context | DiffSpanStyle::Unchanged => theme.diff_context_bg,
            DiffSpanStyle::Separator => theme.diff_separator_bg,
        },
    }
}

pub fn line_text_color(
    style: DiffSpanStyle,
    conflict_kind: ConflictLineKind,
    theme: &Theme,
) -> u32 {
    match conflict_kind {
        ConflictLineKind::Start => theme.diff_conflict_header_fg,
        ConflictLineKind::End | ConflictLineKind::Section => theme.diff_conflict_section_fg,
        ConflictLineKind::Added => theme.diff_text_added,
        ConflictLineKind::Removed => theme.diff_text_removed,
        _ => match style {
            DiffSpanStyle::Added => theme.diff_text_added,
            DiffSpanStyle::Removed => theme.diff_text_removed,
            _ => theme.diff_text_context,
        },
    }
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
        .h(px(ROW_HEIGHT))
        .bg(bg)
}

fn gutter_cell(text: String, theme: &Theme) -> Div {
    div()
        .flex()
        .items_center()
        .justify_end()
        .flex_none()
        .w(px(GUTTER_NUMBER_WIDTH))
        .h(px(ROW_HEIGHT))
        .pl(px(2.))
        .pr(px(5.))
        .text_color(rgb(theme.diff_gutter_fg))
        .bg(rgb(theme.diff_gutter_bg))
        .line_height(px(ROW_HEIGHT))
        .child(SharedString::from(text))
}

fn conflict_label(line: &DiffLine) -> Option<String> {
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

fn separator_gutter(theme: &Theme) -> AnyElement {
    div()
        .w(px(GUTTER_WIDTH))
        .h(px(ROW_HEIGHT))
        .bg(rgb(theme.diff_separator_bg))
        .into_any_element()
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
        .h(px(ROW_HEIGHT))
        .bg(rgb(bg))
        .font_family(fonts::mono())
        .text_size(px(11.))
        .line_height(px(ROW_HEIGHT))
        .text_color(rgb(theme.diff_text_dim))
        .px(px(20.))
        .child(SharedString::from(label))
}

pub fn tag_for_hunk(hunk: &DiffHunk, theme: &Theme) -> (&'static str, u32, u32) {
    match hunk.hunk_type {
        HunkType::Added => ("Added", theme.tag_added_bg, theme.tag_added_fg),
        HunkType::Removed => ("Removed", theme.tag_removed_bg, theme.tag_removed_fg),
        HunkType::Modified => ("Modified", theme.tag_modified_bg, theme.tag_modified_fg),
        HunkType::Renamed => ("Renamed", theme.tag_renamed_bg, theme.tag_renamed_fg),
    }
}

#[cfg(test)]
mod tests;
