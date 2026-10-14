use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
    Window, div, px,
};

use super::SettingsView;
use super::dropdown::dropdown_button;
use super::font_preview::font_preview;
use super::shared::{field_row, form_group, form_section, stepper};
use crate::app::config::{self, AppConfig};
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::ui::icons::glyph;
use crate::ui::primitives::{boolean_toggle_button, button};

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
    let ligatures = if has_ligatures {
        toggle
    } else {
        div().opacity(0.45).child(toggle).into_any_element()
    };
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(24.))
        .child(form_section(
            "Interface Font",
            [
                field_row(
                    glyph::CASE_SENSITIVE,
                    "Family",
                    dropdown_button(
                        "interface-font-family",
                        fonts::interface_preference_label(&cfg.interface_font_family),
                        t,
                        cx,
                    ),
                    t,
                ),
                field_row(
                    glyph::A_LARGE_SMALL,
                    "Size",
                    stepper(
                        "interface-font-size",
                        format!("{:.0}pt", cfg.interface_font_size()),
                        cfg.interface_font_size(),
                        AppConfig::MIN_FONT_SIZE..=AppConfig::MAX_FONT_SIZE,
                        AppConfig::adjust_interface_font_size,
                        t,
                    ),
                    t,
                ),
            ],
            t,
        ))
        .child(form_section(
            "Code Font",
            [
                field_row(
                    glyph::CASE_SENSITIVE,
                    "Family",
                    dropdown_button(
                        "code-font-family",
                        fonts::mono_preference_label(&cfg.code_font_family),
                        t,
                        cx,
                    ),
                    t,
                ),
                field_row(
                    glyph::A_LARGE_SMALL,
                    "Size",
                    stepper(
                        "code-font-size",
                        format!("{:.0}pt", cfg.code_font_size()),
                        cfg.code_font_size(),
                        AppConfig::MIN_FONT_SIZE..=AppConfig::MAX_FONT_SIZE,
                        AppConfig::adjust_code_font_size,
                        t,
                    ),
                    t,
                ),
                field_row(
                    glyph::ARROW_UP_DOWN,
                    "Line spacing",
                    stepper(
                        "code-line-spacing",
                        format!("{:.1}", cfg.code_line_spacing),
                        cfg.code_line_spacing,
                        AppConfig::MIN_LINE_SPACING..=AppConfig::MAX_LINE_SPACING,
                        |cfg, sign| cfg.adjust_line_spacing(sign * 0.1),
                        t,
                    ),
                    t,
                ),
                field_row(glyph::LIGATURE, "Ligatures", ligatures, t),
            ],
            t,
        ))
        .child(form_section("Preview", [font_preview(t, t, cx)], t))
        .child(form_group(
            [div()
                .flex()
                .py(px(10.))
                .child(
                    button("setting-reset-fonts", "Reset Fonts", t, false)
                        .debug_selector(|| "setting-reset-fonts".to_owned())
                        .on_click(|_, _, cx| config::update(cx, AppConfig::reset_fonts)),
                )
                .into_any_element()],
            t,
        ))
        .into_any_element()
}
