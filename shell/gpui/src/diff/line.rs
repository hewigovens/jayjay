use gpui::{AnyElement, Div, IntoElement, ParentElement, SharedString, Styled, div, px, rgb};
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
/// `gutter_row` at the same index in `unified_body`.
pub fn content_row(line: &DiffLine, theme: &Theme, find_query: Option<&str>) -> AnyElement {
    if line.style == DiffSpanStyle::Separator {
        return separator_content(line, theme);
    }
    let (bg, base_text_fg) = match line.style {
        DiffSpanStyle::Added => (theme.diff_added_bg, theme.diff_text_added),
        DiffSpanStyle::Removed => (theme.diff_removed_bg, theme.diff_text_removed),
        DiffSpanStyle::Context | DiffSpanStyle::Unchanged => {
            (theme.diff_context_bg, theme.diff_text_context)
        }
        DiffSpanStyle::Separator => unreachable!("handled above"),
    };

    let mut text_row = div()
        .flex()
        .flex_row()
        .flex_1()
        .min_w_0()
        .h(px(ROW_HEIGHT));
    for span in &line.spans {
        text_row = text_row.child(span_element(
            span,
            base_text_fg,
            line.style,
            theme,
            find_query,
        ));
    }

    div()
        .flex()
        .flex_row()
        .w_full()
        .h(px(ROW_HEIGHT))
        .bg(rgb(bg))
        .font_family(fonts::mono())
        .text_size(px(12.))
        .line_height(px(ROW_HEIGHT))
        .child(text_row)
        .into_any_element()
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

fn separator_content(line: &DiffLine, theme: &Theme) -> AnyElement {
    let label: String = line.spans.iter().map(|s| s.text.as_str()).collect();
    let label = if label.is_empty() {
        String::from("…")
    } else {
        label
    };
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(ROW_HEIGHT))
        .bg(rgb(theme.diff_separator_bg))
        .font_family(fonts::mono())
        .text_size(px(11.))
        .line_height(px(ROW_HEIGHT))
        .text_color(rgb(theme.diff_text_dim))
        .px(px(20.))
        .child(SharedString::from(label))
        .into_any_element()
}

pub fn tag_for_hunk(hunk: &DiffHunk, theme: &Theme) -> (&'static str, u32, u32) {
    match hunk.hunk_type {
        HunkType::Added => ("added", theme.tag_added_bg, theme.tag_added_fg),
        HunkType::Removed => ("removed", theme.tag_removed_bg, theme.tag_removed_fg),
        HunkType::Modified => ("modified", theme.tag_modified_bg, theme.tag_modified_fg),
        HunkType::Renamed => ("renamed", theme.tag_renamed_bg, theme.tag_renamed_fg),
    }
}
