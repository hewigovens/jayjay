use crate::app::config::{AppConfig, AppearanceMode};
use gpui::{
    AnyElement, ClickEvent, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_i18n::settings;

use super::SettingsView;
use super::diff_palette::diff_palette_section;
use super::dropdown::dropdown_button;
use super::shared::{ROW_FONT_SIZE, field_row, form_group};
use crate::app::config;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::glyph;

pub(super) fn appearance_section(
    view: &SettingsView,
    cfg: &AppConfig,
    t: &Theme,
    cx: &mut gpui::Context<SettingsView>,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(24.))
        .child(form_group(
            [field_row(
                glyph::CONTRAST,
                settings::page_appearance(),
                appearance_segmented(cfg.appearance, t),
                t,
            )],
            t,
        ))
        .child(form_group(
            [field_row(
                glyph::GLOBE,
                settings::language(),
                dropdown_button("language", language_label(cfg), t, cx),
                t,
            )],
            t,
        ))
        .child(diff_palette_section(view, cfg, t, cx))
        .into_any_element()
}

fn language_label(cfg: &AppConfig) -> String {
    cfg.language
        .as_deref()
        .and_then(|tag| jayjay_i18n::Language::all().find(|language| language.tag == tag))
        .map_or_else(settings::language_system, |language| {
            language.name.to_owned()
        })
}

fn appearance_segmented(current: AppearanceMode, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .gap(px(1.))
        .p(px(2.))
        .rounded_md()
        .bg(rgb(t.toggle_inactive_bg))
        .child(appearance_option(
            AppearanceMode::System,
            settings::appearance_system(),
            current,
            "appearance-system",
            t,
        ))
        .child(appearance_option(
            AppearanceMode::Light,
            settings::appearance_light(),
            current,
            "appearance-light",
            t,
        ))
        .child(appearance_option(
            AppearanceMode::Dark,
            settings::appearance_dark(),
            current,
            "appearance-dark",
            t,
        ))
        .into_any_element()
}

fn appearance_option(
    mode: AppearanceMode,
    label: String,
    current: AppearanceMode,
    id: &'static str,
    t: &Theme,
) -> AnyElement {
    let active = mode == current;
    let (bg, fg) = if active {
        (t.toggle_active_bg, t.toggle_active_fg)
    } else {
        (t.toggle_inactive_bg, t.toggle_inactive_fg)
    };
    div()
        .id(SharedString::from(id))
        .px(px(10.))
        .py(px(3.))
        .rounded_md()
        .bg(rgb(bg))
        .text_size(ui_font_size(ROW_FONT_SIZE))
        .text_color(rgb(fg))
        .cursor_pointer()
        .on_click(move |_ev: &ClickEvent, _w, cx| {
            config::update(cx, move |c| c.appearance = mode);
        })
        .child(label)
        .into_any_element()
}
