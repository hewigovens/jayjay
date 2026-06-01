use gpui::{AnyElement, Entity, IntoElement, ParentElement, SharedString, Styled, div, px, rgb};

use crate::app::theme::Theme;
use crate::ui::icons::{self, glyph};
use crate::ui::text_area::TextArea;

pub(super) fn header(count: usize, filter: Entity<TextArea>, t: &Theme) -> AnyElement {
    let count_label = if count == 1 {
        "1 bookmark".to_owned()
    } else {
        format!("{count} bookmarks")
    };
    div()
        .flex()
        .flex_col()
        .gap(px(10.))
        .px(px(16.))
        .py(px(12.))
        .bg(rgb(t.header_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(icons::icon(glyph::BOOKMARK, 15., t.fg_dim))
                .child(
                    div()
                        .text_size(px(14.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child("Bookmarks"),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(rgb(t.fg_dim))
                        .child(SharedString::from(count_label)),
                ),
        )
        .child(filter)
        .into_any_element()
}

pub(super) fn placeholder(message: &str, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_1()
        .items_center()
        .justify_center()
        .text_size(px(12.))
        .text_color(rgb(t.fg_dim))
        .child(SharedString::from(message.to_owned()))
        .into_any_element()
}

pub(super) fn placeholder_err(message: &SharedString, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_1()
        .items_center()
        .justify_center()
        .px(px(24.))
        .text_size(px(12.))
        .text_color(rgb(t.error_fg))
        .child(message.clone())
        .into_any_element()
}
