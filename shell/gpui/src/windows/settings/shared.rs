use gpui::{
    AnyElement, ClickEvent, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use crate::app::config;
use crate::app::theme::Theme;
use crate::ui::icons::{self, glyph};

pub(super) fn section_title(text: &'static str, t: &Theme) -> impl IntoElement {
    div()
        .text_size(px(18.))
        .text_color(rgb(t.fg))
        .pb(px(4.))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(text)
}

pub(super) fn field_row(
    label: &'static str,
    value: AnyElement,
    hint: &'static str,
    t: &Theme,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(4.))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .gap(px(12.))
                .child(div().text_size(px(12.)).text_color(rgb(t.fg)).child(label))
                .child(value),
        )
        .child(
            div()
                .text_size(px(11.))
                .text_color(rgb(t.fg_faint))
                .child(hint),
        )
        .into_any_element()
}

pub(super) fn current_value(value: &str, t: &Theme) -> AnyElement {
    div()
        .text_size(px(12.))
        .text_color(rgb(t.fg_dim))
        .child(SharedString::from(value.to_owned()))
        .into_any_element()
}

pub(super) fn toggle_field(
    label: &'static str,
    active: bool,
    hint: &'static str,
    mutate: fn(&mut crate::app::config::AppConfig),
    id: &'static str,
    t: &Theme,
) -> AnyElement {
    let (bg, fg) = if active {
        (t.toggle_active_bg, t.toggle_active_fg)
    } else {
        (t.toggle_inactive_bg, t.toggle_inactive_fg)
    };
    let glyph_str = if active { glyph::CHECK } else { glyph::DOT };
    let value_label = if active { "On" } else { "Off" };

    let value = div()
        .id(SharedString::from(format!("setting-{id}")))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.))
        .px(px(10.))
        .py(px(3.))
        .rounded_sm()
        .bg(rgb(bg))
        .text_size(px(11.))
        .text_color(rgb(fg))
        .cursor_pointer()
        .on_click(move |_ev: &ClickEvent, _w, cx| {
            config::update(cx, mutate);
        })
        .child(icons::icon(glyph_str, 12., fg))
        .child(value_label)
        .into_any_element();

    field_row(label, value, hint, t)
}
