use gpui::{Div, ParentElement, SharedString, Styled, div, px, rgb};
use jayjay_core::diff::side_by_side::SideBySideRow;
use jayjay_core::diff::{DiffSpan, DiffSpanStyle};

use crate::app::fonts;
use crate::app::theme::Theme;

use super::line::{GUTTER_NUMBER_WIDTH, ROW_HEIGHT};
use super::spans::span_element;

const SBS_LINE_NO_WIDTH: f32 = GUTTER_NUMBER_WIDTH;
pub const SBS_GUTTER_WIDTH: f32 = SBS_LINE_NO_WIDTH;

pub fn sbs_row_is_separator(row: &SideBySideRow) -> bool {
    row.old.style == DiffSpanStyle::Separator
}

fn sbs_separator_label(row: &SideBySideRow) -> String {
    let label: String = row.old.spans.iter().map(|s| s.text.as_str()).collect();
    if label.is_empty() {
        String::from("…")
    } else {
        label
    }
}

pub fn sbs_old_gutter(row: &SideBySideRow, theme: &Theme) -> Div {
    if sbs_row_is_separator(row) {
        return separator_gutter(theme);
    }
    side_gutter(row.old.line_no.clone(), row.old.style, theme)
}

pub fn sbs_new_gutter(row: &SideBySideRow, theme: &Theme) -> Div {
    if sbs_row_is_separator(row) {
        return separator_gutter(theme);
    }
    side_gutter(row.new.line_no.clone(), row.new.style, theme)
}

pub fn sbs_old_content(row: &SideBySideRow, theme: &Theme, find_query: Option<&str>) -> Div {
    if sbs_row_is_separator(row) {
        return separator_content(sbs_separator_label(row), theme);
    }
    side_content(&row.old.spans, row.old.style, theme, find_query)
}

pub fn sbs_new_content(row: &SideBySideRow, theme: &Theme, find_query: Option<&str>) -> Div {
    if sbs_row_is_separator(row) {
        return separator_content(sbs_separator_label(row), theme);
    }
    side_content(&row.new.spans, row.new.style, theme, find_query)
}

fn side_gutter(line_no: String, style: DiffSpanStyle, theme: &Theme) -> Div {
    let (bg, _) = side_colors(style, theme);
    div()
        .flex()
        .flex_row()
        .w(px(SBS_GUTTER_WIDTH))
        .h(px(ROW_HEIGHT))
        .bg(rgb(bg))
        .font_family(fonts::mono())
        .text_size(px(12.))
        .line_height(px(ROW_HEIGHT))
        .child(
            div()
                .flex()
                .items_center()
                .justify_end()
                .flex_none()
                .w(px(SBS_LINE_NO_WIDTH))
                .h(px(ROW_HEIGHT))
                .pl(px(2.))
                .pr(px(5.))
                .text_color(rgb(theme.diff_gutter_fg))
                .bg(rgb(theme.diff_gutter_bg))
                .line_height(px(ROW_HEIGHT))
                .child(SharedString::from(line_no)),
        )
}

fn side_content(
    spans: &[DiffSpan],
    style: DiffSpanStyle,
    theme: &Theme,
    find_query: Option<&str>,
) -> Div {
    let (bg, _) = side_colors(style, theme);
    let base_text_fg = side_text_fg(style, theme);

    let mut text_row = div().flex().flex_row().flex_1().min_w_0().h(px(ROW_HEIGHT));
    for span in spans {
        text_row = text_row.child(span_element(span, base_text_fg, style, theme, find_query));
    }

    div()
        .flex()
        .flex_row()
        .flex_1()
        .min_w_0()
        .h(px(ROW_HEIGHT))
        .bg(rgb(bg))
        .font_family(fonts::mono())
        .text_size(px(12.))
        .line_height(px(ROW_HEIGHT))
        .child(text_row)
}

fn side_colors(style: DiffSpanStyle, theme: &Theme) -> (u32, u32) {
    match style {
        DiffSpanStyle::Added => (theme.diff_added_bg, theme.diff_gutter_added_fg),
        DiffSpanStyle::Removed => (theme.diff_removed_bg, theme.diff_gutter_removed_fg),
        _ => (theme.diff_context_bg, theme.diff_gutter_fg),
    }
}

fn side_text_fg(style: DiffSpanStyle, theme: &Theme) -> u32 {
    match style {
        DiffSpanStyle::Added => theme.diff_text_added,
        DiffSpanStyle::Removed => theme.diff_text_removed,
        _ => theme.diff_text_context,
    }
}

fn separator_gutter(theme: &Theme) -> Div {
    div()
        .w(px(SBS_GUTTER_WIDTH))
        .h(px(ROW_HEIGHT))
        .bg(rgb(theme.diff_separator_bg))
}

fn separator_content(label: String, theme: &Theme) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .flex_1()
        .min_w_0()
        .h(px(ROW_HEIGHT))
        .bg(rgb(theme.diff_separator_bg))
        .font_family(fonts::mono())
        .text_size(px(11.))
        .line_height(px(ROW_HEIGHT))
        .text_color(rgb(theme.diff_text_dim))
        .px(px(20.))
        .child(SharedString::from(label))
}
