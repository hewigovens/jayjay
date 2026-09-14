use gpui::{
    AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use super::{SettingsSection, SettingsView};
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons;

pub(super) fn sidebar(
    active: SettingsSection,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    div()
        .id("settings-sidebar")
        .flex()
        .flex_col()
        .flex_none()
        .w(px(190.))
        .h_full()
        .overflow_y_scroll()
        .bg(rgb(t.sidebar_bg))
        .px(px(8.))
        .py(px(12.))
        .children(SettingsSection::ALL.into_iter().map(|section| {
            let label = section.label();
            div()
                .id(SharedString::from(format!("settings-nav-{label}")))
                .debug_selector(move || format!("settings-nav-{label}"))
                .flex()
                .flex_none()
                .items_center()
                .gap(px(8.))
                .px(px(8.))
                .py(px(8.))
                .rounded(px(6.))
                .bg(rgb(if section == active {
                    t.selected_bg
                } else {
                    t.sidebar_bg
                }))
                .text_size(ui_font_size(13.))
                .text_color(rgb(t.fg))
                .cursor_pointer()
                .hover(|s| {
                    s.bg(rgb(if section == active {
                        t.selected_bg
                    } else {
                        t.row_alt_bg
                    }))
                })
                .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                    if this.section == section {
                        return;
                    }
                    this.section = section;
                    this.open_dropdown = None;
                    this.scroll.set_offset(Default::default());
                    match section {
                        SettingsSection::Integrations => this.ensure_tools_loaded(cx),
                        SettingsSection::Jujutsu => this.ensure_jj_config_loaded(cx),
                        _ => {}
                    }
                    cx.notify();
                }))
                .child(
                    div()
                        .flex()
                        .flex_none()
                        .items_center()
                        .justify_center()
                        .size(px(20.))
                        .rounded(px(5.))
                        .bg(rgb(section.color(t)))
                        .child(icons::icon(section.glyph(), 12., t.detail_bg)),
                )
                .child(label)
        }))
        .into_any_element()
}
