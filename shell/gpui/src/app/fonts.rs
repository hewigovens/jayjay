use std::sync::{Arc, OnceLock, RwLock};

use font_kit::source::SystemSource;
use gpui::{App, FontFeatures, Pixels, Styled, TextRun, Window, black, font, px};
use jayjay_core::{CodeFontChoices, font_family_choices, mono_font_family};

use crate::app::config::AppConfig;
use crate::app::theme::Theme;

mod platform;

/// GPUI has no app-wide default font, so window roots and code elements read the configured fonts from here.
struct ActiveFonts {
    interface: String,
    code: String,
    code_features: FontFeatures,
}

impl Default for ActiveFonts {
    fn default() -> Self {
        Self {
            interface: SYSTEM_UI_FONT.to_owned(),
            code: default_mono().to_owned(),
            code_features: FontFeatures::default(),
        }
    }
}

static ACTIVE: OnceLock<RwLock<ActiveFonts>> = OnceLock::new();
static DEFAULT_MONO: OnceLock<String> = OnceLock::new();

const SYSTEM_UI_FONT: &str = ".SystemUIFont";
pub(crate) const SYSTEM_PREFERENCE: &str = "system";

fn with_active<R>(read: impl FnOnce(&ActiveFonts) -> R) -> R {
    match ACTIVE.get_or_init(Default::default).read() {
        Ok(fonts) => read(&fonts),
        Err(_) => read(&ActiveFonts::default()),
    }
}

pub(crate) fn mono() -> String {
    with_active(|fonts| fonts.code.clone())
}

pub(crate) fn interface() -> String {
    with_active(|fonts| fonts.interface.clone())
}

pub(crate) trait CodeText: Styled + Sized {
    fn code_font(self) -> Self {
        let (family, features) =
            with_active(|fonts| (fonts.code.clone(), fonts.code_features.clone()));
        self.font_family(family).font_features(features)
    }

    fn code_text(self, base: f32, theme: &Theme) -> Self {
        self.code_font()
            .text_size(px(theme.scaled_code_font_size(base)))
    }
}

impl<E: Styled> CodeText for E {}

fn default_mono() -> &'static str {
    DEFAULT_MONO.get_or_init(platform::platform_default_mono)
}

// Falls back to ~7.2 px (SF Mono / Menlo at 12 px) on measurement failure.
pub fn mono_advance(cx: &App, size: Pixels) -> Pixels {
    let font_id = cx.text_system().resolve_font(&font(mono()));
    cx.text_system()
        .ch_advance(font_id, size)
        .unwrap_or(px(7.2))
}

pub fn ui_text_width(cx: &App, text: &str, size: Pixels) -> Pixels {
    let font_id = cx.text_system().resolve_font(&font(interface()));
    text.chars()
        .map(|ch| cx.text_system().layout_width(font_id, size, ch))
        .fold(px(0.), |acc, width| acc + width)
}

pub(crate) fn sync_from_config(config: &AppConfig) {
    let next = ActiveFonts {
        interface: pick(&[config.interface_font_family.trim()])
            .unwrap_or_else(|| SYSTEM_UI_FONT.to_owned()),
        code: resolve_preference(&config.code_font_family),
        code_features: if config.code_ligatures {
            FontFeatures::default()
        } else {
            no_ligatures()
        },
    };
    if let Ok(mut fonts) = ACTIVE.get_or_init(Default::default).write() {
        *fonts = next;
    }
}

fn no_ligatures() -> FontFeatures {
    FontFeatures(Arc::new(vec![("calt".into(), 0), ("liga".into(), 0)]))
}

pub(crate) fn code_font_has_ligatures(window: &Window) -> bool {
    const PROBE: &str = "-> => != == === <= >= :: www fi fl";
    let glyphs = |features: FontFeatures| {
        let run = TextRun {
            len: PROBE.len(),
            font: gpui::Font {
                features,
                ..font(mono())
            },
            color: black(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        window
            .text_system()
            .layout_line(PROBE, px(12.), &[run], None)
            .runs
            .iter()
            .flat_map(|run| run.glyphs.iter().map(|glyph| glyph.id))
            .collect::<Vec<_>>()
    };
    glyphs(FontFeatures::default()) != glyphs(no_ligatures())
}

pub(crate) fn interface_font_choices() -> Vec<String> {
    font_family_choices(platform::user_families())
}

pub(crate) fn interface_preference_label(preference: &str) -> String {
    if preference.trim().is_empty() {
        "System".to_owned()
    } else {
        preference.to_owned()
    }
}

pub(crate) fn interface_font_is_installed(preference: &str) -> bool {
    preference.trim().is_empty() || pick(&[preference.trim()]).is_some()
}

pub(crate) fn code_font_is_installed(preference: &str) -> bool {
    let family = mono_font_family(preference);
    family.is_empty() || pick(&[&family]).is_some()
}

pub(crate) fn code_font_choices() -> CodeFontChoices {
    CodeFontChoices::new(platform::monospaced_families())
}

pub(crate) fn mono_preference_id(preference: &str) -> String {
    family_or(preference, SYSTEM_PREFERENCE)
}

pub(crate) fn mono_preference_label(preference: &str) -> String {
    family_or(preference, "System default")
}

fn family_or(preference: &str, system: &str) -> String {
    let family = mono_font_family(preference);
    if family.is_empty() {
        system.to_owned()
    } else {
        family
    }
}

fn resolve_preference(preference: &str) -> String {
    let family = mono_font_family(preference);
    if family.is_empty() {
        return default_mono().to_owned();
    }
    pick(&[&family]).unwrap_or_else(|| default_mono().to_owned())
}

fn pick(candidates: &[&str]) -> Option<String> {
    let source = SystemSource::new();
    candidates
        .iter()
        .find(|name| source.select_family_by_name(name).is_ok())
        .map(|name| (*name).to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_system_preferences_use_the_system_font() {
        for preference in ["", "system"] {
            assert_eq!(mono_preference_id(preference), SYSTEM_PREFERENCE);
            assert_eq!(mono_preference_label(preference), "System default");
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn system_default_uses_appkit_monospace_token_on_macos() {
        assert_eq!(resolve_preference(""), ".AppleSystemUIFontMonospaced");
    }
}
