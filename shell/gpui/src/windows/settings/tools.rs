use crate::app::config::AppConfig;
use crate::app::tools::{EDITOR_OPTIONS, TERMINAL_OPTIONS};
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, SharedString, Styled, div, px, rgb,
};

use super::SettingsView;
use super::shared::{current_value, field_row, section_title};
use crate::app::theme::Theme;
use crate::ui::icons::{self, glyph};

pub(super) fn tools_section(
    cfg: &AppConfig,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(section_title("Tools", t))
        .child(field_row(
            "External editor",
            dropdown_button("editor", EDITOR_OPTIONS, &cfg.tools.external_editor, t, cx),
            "Used by 'Open in Editor' actions.",
            t,
        ))
        .child(field_row(
            "Custom editor command",
            current_value(
                if cfg.tools.custom_editor_command.is_empty() {
                    "(none)"
                } else {
                    cfg.tools.custom_editor_command.as_str()
                },
                t,
            ),
            "Required when editor = 'Custom'. Edit ~/.config/jayjay/config.toml.",
            t,
        ))
        .child(field_row(
            "Terminal",
            dropdown_button("terminal", TERMINAL_OPTIONS, &cfg.tools.terminal, t, cx),
            "Used by 'Open in Terminal'.",
            t,
        ))
        .into_any_element()
}

fn dropdown_button(
    field_id: &'static str,
    options: &'static [(&'static str, &'static str)],
    current: &str,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let label = options
        .iter()
        .find(|(id, _)| *id == current)
        .map(|(_, l)| *l)
        .unwrap_or(current);
    div()
        .id(SharedString::from(format!("dd-btn-{field_id}")))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .min_w(px(180.))
        .justify_between()
        .px(px(10.))
        .py(px(4.))
        .rounded_sm()
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(t.toggle_inactive_bg))
        .text_size(px(11.))
        .text_color(rgb(t.fg))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |view, ev: &MouseDownEvent, _w, cx| {
                view.open_dropdown(SharedString::from(field_id), ev.position, cx);
            }),
        )
        .child(SharedString::from(label.to_owned()))
        .child(icons::icon(glyph::CARET_DOWN, 10., t.fg_faint))
        .into_any_element()
}
