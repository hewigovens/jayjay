use gpui::{AnyElement, Div, IntoElement, ParentElement, SharedString, Styled, div, px, rgb};
use jayjay_core::diff::side_by_side::SideBySideRow;
use jayjay_core::diff::{DiffSpan, DiffSpanStyle};

use crate::app::fonts;
use crate::app::theme::Theme;

use super::spans::span_element;

pub fn side_by_side_row(
    row: &SideBySideRow,
    theme: &Theme,
    find_query: Option<&str>,
) -> AnyElement {
    if row.old_style == DiffSpanStyle::Separator {
        let label: String = row.old_spans.iter().map(|s| s.text.as_str()).collect();
        let label = if label.is_empty() {
            String::from("…")
        } else {
            label
        };
        return div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h(px(18.))
            .bg(rgb(theme.diff_separator_bg))
            .font_family(fonts::mono())
            .text_size(px(11.))
            .line_height(px(18.))
            .text_color(rgb(theme.diff_text_dim))
            .px(px(40.))
            .child(SharedString::from(label))
            .into_any_element();
    }

    div()
        .flex()
        .flex_row()
        .w_full()
        .h(px(18.))
        .font_family(fonts::mono())
        .text_size(px(12.))
        .line_height(px(18.))
        .child(side_cell(
            row.old_line_no.clone(),
            row.old_marker.clone(),
            &row.old_spans,
            row.old_style,
            theme,
            find_query,
        ))
        .child(div().w(px(1.)).h(px(18.)).bg(rgb(theme.border)))
        .child(side_cell(
            row.new_line_no.clone(),
            row.new_marker.clone(),
            &row.new_spans,
            row.new_style,
            theme,
            find_query,
        ))
        .into_any_element()
}

fn side_cell(
    line_no: String,
    marker: String,
    spans: &[DiffSpan],
    style: DiffSpanStyle,
    theme: &Theme,
    find_query: Option<&str>,
) -> Div {
    let (bg, marker_fg, base_text_fg) = match style {
        DiffSpanStyle::Added => (
            theme.diff_added_bg,
            theme.diff_gutter_added_fg,
            theme.diff_text_added,
        ),
        DiffSpanStyle::Removed => (
            theme.diff_removed_bg,
            theme.diff_gutter_removed_fg,
            theme.diff_text_removed,
        ),
        _ => (
            theme.diff_context_bg,
            theme.diff_gutter_fg,
            theme.diff_text_context,
        ),
    };

    let mut text_row = div().flex().flex_row().flex_1().min_w_0().h(px(18.));
    for span in spans {
        text_row = text_row.child(span_element(span, base_text_fg, style, theme, find_query));
    }

    div()
        .flex()
        .flex_row()
        .flex_1()
        .min_w_0()
        .h(px(18.))
        .bg(rgb(bg))
        .line_height(px(18.))
        .child(
            div()
                .flex_none()
                .w(px(48.))
                .h(px(18.))
                .px(px(6.))
                .text_color(rgb(theme.diff_gutter_fg))
                .bg(rgb(theme.diff_gutter_bg))
                .line_height(px(18.))
                .child(SharedString::from(line_no)),
        )
        .child(
            div()
                .flex_none()
                .w(px(14.))
                .h(px(18.))
                .text_color(rgb(marker_fg))
                .line_height(px(18.))
                .child(SharedString::from(marker)),
        )
        .child(text_row)
}
