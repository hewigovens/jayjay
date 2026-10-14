use gpui::{AnyElement, Context, IntoElement, ParentElement, Styled, WindowAppearance, div, px};

use super::SettingsView;
use super::dropdown::dropdown_button;
use super::font_preview::font_preview;
use super::shared::{field_row, subsection_title};
use crate::app::config::{AppConfig, AppearanceMode};
use crate::app::theme::Theme;

pub(super) fn diff_palette_section(
    view: &SettingsView,
    cfg: &AppConfig,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let palette_row = |label, field_id, is_dark, cx: &mut Context<SettingsView>| {
        field_row(
            label,
            dropdown_button(field_id, cfg.diff.palette.resolve(is_dark).name, t, cx),
            "",
            t,
        )
    };
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(16.))
        .child(subsection_title("Colors", t))
        .child(palette_row("Light theme", "diff-light-palette", false, cx))
        .child(palette_row("Dark theme", "diff-dark-palette", true, cx))
        .child(subsection_title("Preview", t))
        .child(font_preview(
            t,
            &preview_theme(cfg, view.diff_preview_dark.unwrap_or(t.is_dark)),
            cx,
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
