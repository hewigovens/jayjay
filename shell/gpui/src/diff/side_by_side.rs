use gpui::{Div, ParentElement, SharedString, Styled, div, px, rgb};
use jayjay_core::diff::side_by_side::{RowSide, SideBySideRow};
use jayjay_core::diff::{ConflictLineKind, DiffSpanStyle};

use crate::app::fonts::CodeText as _;
use crate::app::theme::Theme;

use super::line::{
    CODE_INSET, GUTTER_NUMBER_WIDTH, conflict_stripe_overlay, gutter_cell, line_bg_color,
    line_text_color,
};
use super::spans::span_element;

pub const SBS_GUTTER_WIDTH: f32 = GUTTER_NUMBER_WIDTH;

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
    side_gutter(
        row.old.line_no.clone(),
        row.old.style,
        row.old.conflict_kind,
        theme,
    )
}

pub fn sbs_new_gutter(row: &SideBySideRow, theme: &Theme) -> Div {
    if sbs_row_is_separator(row) {
        return separator_gutter(theme);
    }
    side_gutter(
        row.new.line_no.clone(),
        row.new.style,
        row.new.conflict_kind,
        theme,
    )
}

pub fn sbs_old_content(row: &SideBySideRow, theme: &Theme, find_query: Option<&str>) -> Div {
    if sbs_row_is_separator(row) {
        return separator_content(sbs_separator_label(row), theme);
    }
    side_content(&row.old, theme, find_query)
}

pub fn sbs_new_content(row: &SideBySideRow, theme: &Theme, find_query: Option<&str>) -> Div {
    if sbs_row_is_separator(row) {
        return separator_content(sbs_separator_label(row), theme);
    }
    side_content(&row.new, theme, find_query)
}

fn side_gutter(
    line_no: String,
    style: DiffSpanStyle,
    conflict_kind: ConflictLineKind,
    theme: &Theme,
) -> Div {
    let bg = line_bg_color(style, conflict_kind, theme);
    div()
        .flex()
        .flex_row()
        .w(px(SBS_GUTTER_WIDTH))
        .h(px(theme.code_line_height()))
        .code_text(12., theme)
        .line_height(px(theme.code_line_height()))
        .child(gutter_cell(line_no, theme, bg))
}

fn side_content(side: &RowSide, theme: &Theme, find_query: Option<&str>) -> Div {
    let bg = line_bg_color(side.style, side.conflict_kind, theme);
    let base_text_fg = line_text_color(side.style, side.conflict_kind, theme);

    let mut text_row = div()
        .flex()
        .flex_row()
        .flex_1()
        .min_w_0()
        .h(px(theme.code_line_height()))
        .pl(px(CODE_INSET));
    for span in &side.spans {
        text_row = text_row.child(span_element(
            span,
            base_text_fg,
            side.style,
            theme,
            find_query,
        ));
    }

    div()
        .flex()
        .flex_row()
        .flex_1()
        .min_w_0()
        .h(px(theme.code_line_height()))
        .bg(rgb(bg))
        .code_text(12., theme)
        .line_height(px(theme.code_line_height()))
        .relative()
        .child(conflict_stripe_overlay(side.conflict_kind, theme))
        .child(text_row)
}

fn separator_gutter(theme: &Theme) -> Div {
    div()
        .w(px(SBS_GUTTER_WIDTH))
        .h(px(theme.code_line_height()))
        .bg(rgb(theme.diff_separator_bg))
}

fn separator_content(label: String, theme: &Theme) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .flex_1()
        .min_w_0()
        .h(px(theme.code_line_height()))
        .bg(rgb(theme.diff_separator_bg))
        .code_font()
        .text_size(px(theme.compact_code_font_size()))
        .line_height(px(theme.code_line_height()))
        .text_color(rgb(theme.diff_text_dim))
        .px(px(20.))
        .child(SharedString::from(label))
}
