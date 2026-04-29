use gpui::{AnyElement, IntoElement, ParentElement, SharedString, Styled, div, px, rgb};

use crate::app::fonts;
use crate::app::theme::Theme;
use crate::ui::icons::{self, glyph};

pub(super) fn render_find_bar(
    query: &str,
    match_count: usize,
    match_current: usize,
    t: &Theme,
) -> AnyElement {
    let display = if query.is_empty() {
        SharedString::from("Type to find…")
    } else {
        SharedString::from(query.to_owned())
    };
    let color = if query.is_empty() { t.fg_faint } else { t.fg };
    let count_label = if query.is_empty() {
        String::new()
    } else if match_count == 0 {
        String::from("No matches")
    } else {
        format!("{} of {}", match_current + 1, match_count)
    };
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(12.))
        .py(px(6.))
        .bg(rgb(t.header_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(icons::icon(glyph::SEARCH, 12., t.fg_dim))
        .child(
            div()
                .flex_1()
                .text_size(px(12.))
                .text_color(rgb(color))
                .font_family(fonts::mono())
                .child(display),
        )
        .child(
            div()
                .text_size(px(10.))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(count_label)),
        )
        .child(
            div()
                .text_size(px(10.))
                .text_color(rgb(t.fg_faint))
                .child("⏎ next · ⇧⏎ prev · Esc"),
        )
        .into_any_element()
}
