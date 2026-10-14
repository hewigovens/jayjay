use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
    Window, div, px,
};

use super::SettingsView;
use super::dropdown::dropdown_button;
use super::font_preview::font_preview;
use super::shared::{field_row, section_title, stepper, subsection_title};
use crate::app::config::{self, AppConfig};
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::ui::primitives::{boolean_toggle_button, button};

const FONT_SIZE_HINT: &str = if cfg!(target_os = "macos") {
    "Press ⌘+ or ⌘- to change the interface and code sizes together, ⌘0 to reset."
} else {
    "Press Ctrl++ or Ctrl+- to change the interface and code sizes together, Ctrl+0 to reset."
};

pub(super) fn fonts_section(
    cfg: &AppConfig,
    t: &Theme,
    window: &Window,
    cx: &mut gpui::Context<SettingsView>,
) -> AnyElement {
    let has_ligatures = fonts::code_font_has_ligatures(window);
    let toggle = boolean_toggle_button(
        "setting-code-ligatures",
        has_ligatures && cfg.code_ligatures,
        t,
        move |_, _, cx| {
            if has_ligatures {
                config::update(cx, |cfg| cfg.code_ligatures = !cfg.code_ligatures);
            }
        },
    );
    let (ligatures, ligatures_hint) = if has_ligatures {
        (toggle, "")
    } else {
        (
            div().opacity(0.45).child(toggle).into_any_element(),
            "This font has no ligatures.",
        )
    };
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(16.))
        .child(section_title("Fonts", t))
        .child(subsection_title("Interface Font", t))
        .child(field_row(
            "Family",
            dropdown_button(
                "interface-font-family",
                fonts::interface_preference_label(&cfg.interface_font_family),
                t,
                cx,
            ),
            if fonts::interface_font_is_installed(&cfg.interface_font_family) {
                "Used for interface text.".to_owned()
            } else {
                format!(
                    "{} is not installed; using System.",
                    cfg.interface_font_family
                )
            },
            t,
        ))
        .child(field_row(
            "Size",
            stepper(
                "interface-font-size",
                format!("{:.0}pt", cfg.interface_font_size()),
                cfg.interface_font_size(),
                AppConfig::MIN_FONT_SIZE..=AppConfig::MAX_FONT_SIZE,
                AppConfig::adjust_interface_font_size,
                t,
            ),
            FONT_SIZE_HINT,
            t,
        ))
        .child(subsection_title("Code Font", t))
        .child(field_row(
            "Family",
            dropdown_button(
                "code-font-family",
                fonts::mono_preference_label(&cfg.code_font_family),
                t,
                cx,
            ),
            if fonts::code_font_is_installed(&cfg.code_font_family) {
                "Used for diffs, editors, revsets, and identifiers.".to_owned()
            } else {
                format!(
                    "{} is not installed; using the system monospace font.",
                    cfg.code_font_family
                )
            },
            t,
        ))
        .child(field_row(
            "Size",
            stepper(
                "code-font-size",
                format!("{:.0}pt", cfg.code_font_size()),
                cfg.code_font_size(),
                AppConfig::MIN_FONT_SIZE..=AppConfig::MAX_FONT_SIZE,
                AppConfig::adjust_code_font_size,
                t,
            ),
            "",
            t,
        ))
        .child(field_row(
            "Line spacing",
            stepper(
                "code-line-spacing",
                format!("{:.1}", cfg.code_line_spacing),
                cfg.code_line_spacing,
                AppConfig::MIN_LINE_SPACING..=AppConfig::MAX_LINE_SPACING,
                |cfg, sign| cfg.adjust_line_spacing(sign * 0.1),
                t,
            ),
            "",
            t,
        ))
        .child(field_row("Ligatures", ligatures, ligatures_hint, t))
        .child(subsection_title("Preview", t))
        .child(font_preview(t, t, cx))
        .child(
            div().flex().child(
                button("setting-reset-fonts", "Reset Fonts", t, false)
                    .debug_selector(|| "setting-reset-fonts".to_owned())
                    .on_click(|_, _, cx| config::update(cx, AppConfig::reset_fonts)),
            ),
        )
        .into_any_element()
}
