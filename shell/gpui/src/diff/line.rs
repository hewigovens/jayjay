use std::ops::Range;

use gpui::{
    AnyElement, Div, IntoElement, ParentElement, Pixels, SharedString, Styled, div, px, rgb, rgba,
};
use jayjay_core::diff::{DiffLine, DiffSpanStyle};
use jayjay_core::{DiffHunk, HunkType};

use crate::app::fonts;
use crate::app::theme::Theme;

use super::spans::span_element;

pub const ROW_HEIGHT: f32 = 18.;
const GUTTER_NUMBER_WIDTH: f32 = 40.;
pub const GUTTER_WIDTH: f32 = GUTTER_NUMBER_WIDTH * 2.;

pub fn gutter_row(line: &DiffLine, theme: &Theme) -> AnyElement {
    if line.style == DiffSpanStyle::Separator {
        return separator_gutter(theme);
    }
    let bg = match line.style {
        DiffSpanStyle::Added => theme.diff_added_bg,
        DiffSpanStyle::Removed => theme.diff_removed_bg,
        DiffSpanStyle::Context | DiffSpanStyle::Unchanged => theme.diff_context_bg,
        DiffSpanStyle::Separator => unreachable!("handled above"),
    };
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
    let (bg, base_text_fg) = match line.style {
        DiffSpanStyle::Added => (theme.diff_added_bg, theme.diff_text_added),
        DiffSpanStyle::Removed => (theme.diff_removed_bg, theme.diff_text_removed),
        DiffSpanStyle::Context | DiffSpanStyle::Unchanged => {
            (theme.diff_context_bg, theme.diff_text_context)
        }
        DiffSpanStyle::Separator => unreachable!("handled above"),
    };

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
        .child(text_row);
    if let Some(cols) = selection_cols {
        row = row.child(selection_overlay(cols, advance, theme));
    }
    row
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
        .flex_none()
        .w(px(GUTTER_NUMBER_WIDTH))
        .h(px(ROW_HEIGHT))
        .px(px(4.))
        .text_color(rgb(theme.diff_gutter_fg))
        .bg(rgb(theme.diff_gutter_bg))
        .line_height(px(ROW_HEIGHT))
        .child(SharedString::from(text))
}

fn separator_gutter(theme: &Theme) -> AnyElement {
    div()
        .w(px(GUTTER_WIDTH))
        .h(px(ROW_HEIGHT))
        .bg(rgb(theme.diff_separator_bg))
        .into_any_element()
}

fn separator_content(line: &DiffLine, theme: &Theme, is_selected: bool) -> Div {
    let label: String = line.spans.iter().map(|s| s.text.as_str()).collect();
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
        HunkType::Added => ("added", theme.tag_added_bg, theme.tag_added_fg),
        HunkType::Removed => ("removed", theme.tag_removed_bg, theme.tag_removed_fg),
        HunkType::Modified => ("modified", theme.tag_modified_bg, theme.tag_modified_fg),
        HunkType::Renamed => ("renamed", theme.tag_renamed_bg, theme.tag_renamed_fg),
    }
}
