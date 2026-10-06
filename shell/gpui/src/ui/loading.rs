use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, AnyElement, Div, IntoElement, ParentElement, SharedString,
    Styled, Transformation, div, percentage, px, rgb, svg,
};

use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons;
use crate::ui::overlay::overlay_layer;

pub(crate) fn loading_label(label: impl Into<SharedString>, t: &Theme) -> Div {
    let spinner = svg()
        .path(icons::REFRESH_CW_SVG)
        .flex_none()
        .w(px(14.))
        .h(px(14.))
        .text_color(rgb(t.fg_dim))
        .with_animation(
            "loading-spinner",
            Animation::new(Duration::from_secs(1)).repeat(),
            |icon, delta| icon.with_transformation(Transformation::rotate(percentage(delta))),
        );
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .text_size(ui_font_size(12.))
        .text_color(rgb(t.fg_dim))
        .child(spinner)
        .child(label.into())
}

pub(crate) fn loading_status(label: impl Into<SharedString>, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_1()
        .size_full()
        .items_center()
        .justify_center()
        .child(loading_label(label, t))
        .into_any_element()
}

/// For loading over content that stays visible.
pub(crate) fn loading_hud(t: &Theme) -> AnyElement {
    let hud = loading_label("Loading…", t)
        .px(px(16.))
        .py(px(12.))
        .rounded_lg()
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(t.header_bg));
    overlay_layer().child(hud).into_any_element()
}
