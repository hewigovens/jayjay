use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, AnyElement, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, Styled, Transformation, div, percentage, px, rgb, rgba, svg,
};

use crate::app::theme::Theme;
use crate::ui::icons;

pub(crate) fn loading_hud(t: &Theme) -> AnyElement {
    let spinner = svg()
        .path(icons::REFRESH_CW_SVG)
        .w(px(16.))
        .h(px(16.))
        .text_color(rgb(t.fg_dim))
        .with_animation(
            "loading-hud-spinner",
            Animation::new(Duration::from_secs(1)).repeat(),
            |icon, delta| icon.with_transformation(Transformation::rotate(percentage(delta))),
        );
    let hud = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(10.))
        .px(px(16.))
        .py(px(12.))
        .rounded_lg()
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(t.header_bg))
        .text_size(px(12.))
        .text_color(rgb(t.fg))
        .child(spinner)
        .child("Loading...");

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
        .occlude()
        .on_mouse_down(MouseButton::Left, |_: &MouseDownEvent, _, _| {})
        .child(hud)
        .into_any_element()
}
