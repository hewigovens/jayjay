use gpui::{AnyElement, Context, IntoElement, ParentElement, Styled, WindowAppearance, div, px};

use super::SettingsView;
use super::dropdown::dropdown_button;
use super::font_preview::font_preview;
use super::shared::{field_row, form_section};
use crate::app::config::{AppConfig, AppearanceMode};
use crate::app::theme::Theme;
use crate::ui::icons::glyph;

pub(super) fn diff_palette_section(
    view: &SettingsView,
    cfg: &AppConfig,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let palette_row = |icon, label, field_id, is_dark, cx: &mut Context<SettingsView>| {
        field_row(
            icon,
            label,
            dropdown_button(field_id, cfg.diff.palette.resolve(is_dark).name, t, cx),
            t,
        )
    };
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(24.))
        .child(form_section(
            "Colors",
            [
                palette_row(glyph::SUN, "Light theme", "diff-light-palette", false, cx),
                palette_row(glyph::MOON, "Dark theme", "diff-dark-palette", true, cx),
            ],
            t,
        ))
        .child(form_section(
            "Preview",
            [font_preview(
                t,
                &preview_theme(cfg, view.diff_preview_dark.unwrap_or(t.is_dark)),
                cx,
            )],
            t,
        ))
        .into_any_element()
}

fn preview_theme(cfg: &AppConfig, is_dark: bool) -> Theme {
    let mode = if is_dark {
        AppearanceMode::Dark
    } else {
        AppearanceMode::Light
    };
    Theme::for_appearance(mode, WindowAppearance::Light, &cfg.diff.palette).with_typography(cfg)
}
