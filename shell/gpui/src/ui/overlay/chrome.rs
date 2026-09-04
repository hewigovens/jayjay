use gpui::{
    Div, FontWeight, InteractiveElement, IntoElement, MouseButton, MouseDownEvent, ParentElement,
    SharedString, Styled, div, px, rgb, rgba,
};

use crate::app::theme::{Theme, ui_font_size};
use crate::ui::primitives::icon_label;

/// Dimmed, occluding full-window layer. Centers its child; callers add the card or sheet.
pub(crate) fn overlay_layer() -> Div {
    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0x00000033))
        // occlude() also swallows scroll-wheel events, or scrolling a modal list scrolls the view underneath.
        .occlude()
        .on_mouse_down(MouseButton::Left, |_: &MouseDownEvent, _, _| {})
}

pub(crate) fn overlay_card(t: &Theme, width: f32) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .w(px(width))
        .max_w_full()
        .px(px(18.))
        .py(px(16.))
        .rounded_lg()
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(t.header_bg))
}

pub(crate) fn overlay_header(
    icon: &'static str,
    icon_color: u32,
    title: impl Into<SharedString>,
    subtitle: impl Into<SharedString>,
    t: &Theme,
) -> Div {
    let subtitle = subtitle.into();
    let mut row = div().flex().flex_row().items_center().child(
        icon_label(icon, title, 16., icon_color)
            .text_size(ui_font_size(14.))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(rgb(t.fg)),
    );
    if !subtitle.is_empty() {
        row = row.child(div().flex_1()).child(
            div()
                .font_family(crate::app::fonts::mono())
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.fg_dim))
                .child(subtitle),
        );
    }
    row
}

pub(crate) fn overlay_actions(cancel: impl IntoElement, primary: impl IntoElement) -> Div {
    div()
        .flex()
        .flex_row()
        .justify_end()
        .gap(px(8.))
        .child(cancel)
        .child(primary)
}
