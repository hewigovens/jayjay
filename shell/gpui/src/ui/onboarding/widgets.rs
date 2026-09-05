use gpui::{InteractiveElement, IntoElement, ParentElement, SharedString, Styled, div, px, rgb};

use crate::app::fonts;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons;
use crate::ui::primitives::copy_icon_button;

pub(super) fn mono_line(text: String, size: f32, color: u32) -> impl IntoElement {
    div()
        .max_w(px(360.))
        .font_family(fonts::mono())
        .text_size(ui_font_size(size))
        .text_color(rgb(color))
        .child(text)
}

pub(super) fn command_row(command: &'static str, t: &Theme) -> impl IntoElement {
    div()
        .id(SharedString::from(format!("onboarding-command-{command}")))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .gap(px(10.))
        .px(px(10.))
        .py(px(8.))
        .rounded_sm()
        .bg(rgb(t.row_alt_bg))
        .text_color(rgb(t.fg))
        .child(
            div()
                .font_family(fonts::mono())
                .text_size(ui_font_size(13.))
                .child(command),
        )
        .child(
            copy_icon_button(
                SharedString::from(format!("onboarding-copy-{command}")),
                command,
                13.,
                24.,
                22.,
                t.fg_dim,
                t,
            )
            .hover(|s| s.bg(rgb(t.selected_bg))),
        )
}

pub(super) fn tip(glyph_str: &'static str, text: &'static str, t: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(10.))
        .text_size(ui_font_size(13.))
        .text_color(rgb(t.fg_dim))
        .child(icons::icon(glyph_str, 16., t.selected_accent))
        .child(text)
}
