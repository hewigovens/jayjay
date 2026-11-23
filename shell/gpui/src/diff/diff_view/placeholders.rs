use gpui::{
    AnyElement, Div, FontWeight, InteractiveElement, IntoElement, ParentElement, SharedString,
    Styled, div, px, rgb,
};

use super::header::{DETAIL_INSET, file_icon_width};
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons;

pub(super) fn placeholder_card(
    glyph_str: &'static str,
    title: impl Into<SharedString>,
    body: impl Into<SharedString>,
    action: Option<AnyElement>,
    t: &Theme,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .size_full()
        .items_center()
        .justify_center()
        .gap(px(10.))
        .px(px(40.))
        .bg(rgb(t.detail_bg))
        .child(icons::icon(glyph_str, 28., t.fg_dim))
        .child(
            div()
                .text_size(ui_font_size(14.))
                .text_color(rgb(t.fg))
                .child(title.into()),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .justify_center()
                .gap(px(4.))
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.fg_dim))
                .child(body.into())
                .children(action),
        )
}

pub(super) fn placeholder(
    title: impl Into<SharedString>,
    body: impl Into<SharedString>,
    t: &Theme,
) -> impl IntoElement {
    div()
        .flex()
        .flex_1()
        .flex_col()
        .size_full()
        .items_center()
        .justify_center()
        .gap(px(6.))
        .bg(rgb(t.detail_bg))
        .child(
            div()
                .text_size(ui_font_size(17.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(t.fg_dim))
                .child(title.into()),
        )
        .child(
            div()
                .text_size(ui_font_size(12.))
                .text_color(rgb(t.fg_faint))
                .child(body.into()),
        )
}

pub(super) fn placeholder_inner(text: impl Into<SharedString>, t: &Theme) -> Div {
    div()
        .flex()
        .size_full()
        .items_center()
        .justify_center()
        .text_color(rgb(t.fg_dim))
        .child(text.into())
}

/// `path` is the file whose header sits above: the icon centres under its file icon, which also starts the title where the path starts.
pub(crate) fn placeholder_row(
    path: &str,
    glyph_str: &'static str,
    title: SharedString,
    detail: Option<SharedString>,
    action: Option<AnyElement>,
    t: &Theme,
) -> AnyElement {
    const ICON_SIZE: f32 = 12.;
    const ICON_GAP: f32 = 6.;
    let icon_width = file_icon_width(path, t);
    div()
        .debug_selector(|| "stack-message".to_owned())
        .flex()
        .flex_col()
        .size_full()
        .justify_center()
        .gap(px(3.))
        .px(px(DETAIL_INSET))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(ICON_GAP))
                .text_size(ui_font_size(12.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(t.fg))
                .child(
                    div()
                        .flex()
                        .flex_none()
                        .justify_center()
                        .w(px(icon_width))
                        .child(icons::icon(glyph_str, ICON_SIZE, t.fg_dim)),
                )
                .child(title)
                .children(action),
        )
        .children(detail.map(|detail| {
            div()
                .pl(px(icon_width + ICON_GAP))
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.fg_dim))
                .line_clamp(2)
                .child(detail)
        }))
        .into_any_element()
}
