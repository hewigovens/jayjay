use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, AnyElement, IntoElement, ParentElement, Styled, Transformation,
    div, percentage, px, rgb, svg,
};

use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons;
use crate::ui::overlay::overlay_layer;

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
        .text_size(ui_font_size(12.))
        .text_color(rgb(t.fg))
        .child(spinner)
        .child("Loading...");

    overlay_layer().child(hud).into_any_element()
}
