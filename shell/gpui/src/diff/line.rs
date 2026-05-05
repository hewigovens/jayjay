use std::ops::Range;

use gpui::{AnyElement, Div, IntoElement, ParentElement, SharedString, Styled, div, px, rgb, rgba};
use jayjay_core::diff::{DiffLine, DiffSpanStyle};
use jayjay_core::{DiffHunk, HunkType};

use crate::app::fonts;
use crate::app::theme::Theme;

use super::spans::span_element;

pub const ROW_HEIGHT: f32 = 18.;
const GUTTER_NUMBER_WIDTH: f32 = 40.;
const GUTTER_PREFIX_WIDTH: f32 = 16.;
/// Total gutter panel width: old line no + new line no + +/- prefix.
pub const GUTTER_WIDTH: f32 = GUTTER_NUMBER_WIDTH * 2. + GUTTER_PREFIX_WIDTH;
/// Glyph width for the monospace font at the diff text size. Calibrated for
/// SF Mono / Menlo at 12px; close enough for column-precision selection on
/// other monospace faces. Refine via `text_system().bounds_for_glyph()` if
/// pixel drift becomes visible on long lines.
pub const MONO_GLYPH_WIDTH: f32 = 7.2;

/// Renders the gutter cells (old/new line numbers + change marker) for one
/// diff line. Pairs with `content_row` at the same index in `unified_body`.
pub fn gutter_row(line: &DiffLine, theme: &Theme) -> AnyElement {
    if line.style == DiffSpanStyle::Separator {
        return separator_gutter(theme);
    }
    let (bg, prefix, prefix_fg) = match line.style {
        DiffSpanStyle::Added => (theme.diff_added_bg, "+", theme.diff_gutter_added_fg),
        DiffSpanStyle::Removed => (theme.diff_removed_bg, "-", theme.diff_gutter_removed_fg),
        DiffSpanStyle::Context | DiffSpanStyle::Unchanged => {
            (theme.diff_context_bg, " ", theme.diff_gutter_fg)
        }
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
        .child(
            div()
                .flex_none()
                .w(px(GUTTER_PREFIX_WIDTH))
                .h(px(ROW_HEIGHT))
                .text_color(rgb(prefix_fg))
                .child(prefix),
        )
        .into_any_element()
}

/// Renders just the content (highlighted spans) for one diff line. Pairs with
/// `gutter_row` at the same index in `unified_body`. `selection_cols` paints a
/// column-precision selected_bg overlay across the row at the given char range.
pub fn content_row(
    line: &DiffLine,
    theme: &Theme,
    find_query: Option<&str>,
    selection_cols: Option<Range<usize>>,
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

    let line_len = line
        .spans
        .iter()
        .map(|s| s.text.chars().count())
        .sum::<usize>();
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
        row = row.child(selection_overlay(cols, line_len, theme));
    }
    row
}

/// Translucent so per-span add/remove backgrounds underneath stay visible.
/// `cols.end == line_len` snaps to the parent's right edge so trailing chars
/// are covered regardless of MONO_GLYPH_WIDTH drift; mid-line uses the
/// hardcode but with bounded visual error. MUST be the row's last child so
/// it paints on top of the text — gpui paints siblings in declaration order.
pub fn selection_overlay(cols: Range<usize>, line_len: usize, theme: &Theme) -> Div {
    let left = cols.start as f32 * MONO_GLYPH_WIDTH;
    let right_offset = line_len.saturating_sub(cols.end) as f32 * MONO_GLYPH_WIDTH;
    let bg = rgba(((theme.selected_bg as u64) << 8) as u32 | 0x66);
    div()
        .absolute()
        .left(px(left))
        .right(px(right_offset))
        .top(px(0.))
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
