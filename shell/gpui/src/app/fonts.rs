use std::sync::{Arc, OnceLock, RwLock};

use font_kit::source::SystemSource;
use gpui::{App, FontFeatures, Pixels, Styled, TextRun, Window, black, font, px};
use jayjay_core::{
    MONO_FONT_OPTIONS, MonoFontOption, SYSTEM_MONO_FONT_ID, mono_font_option,
    mono_font_option_named, my_mono_font_families,
};

use crate::app::config::AppConfig;
use crate::app::theme::Theme;

mod platform;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonoFontChoice {
    pub(crate) id: String,
    pub(crate) title: String,
}

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
    let mut families = platform::user_families();
    families.sort_by_key(|family| family.to_lowercase());
    families.dedup();
    families
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
    match matched_option(preference) {
        Some(option) => option_is_available(option, &SystemSource::new()),
        None => pick(&[preference.trim()]).is_some(),
    }
}

pub(crate) fn mono_font_choices() -> Vec<MonoFontChoice> {
    let source = SystemSource::new();
    MONO_FONT_OPTIONS
        .iter()
        .filter(|option| option_is_available(option, &source))
        .map(|option| MonoFontChoice {
            id: option.id.to_owned(),
            title: mono_option_title(option).to_owned(),
        })
        .collect()
}

pub(crate) fn my_mono_fonts() -> Vec<String> {
    my_mono_font_families(platform::user_mono_families())
}

pub(crate) fn mono_preference_id(preference: &str) -> String {
    matched_option(preference)
        .map(|option| option.id.to_owned())
        .unwrap_or_else(|| {
            if preference.is_empty() {
                SYSTEM_MONO_FONT_ID.to_owned()
            } else {
                preference.to_owned()
            }
        })
}

pub(crate) fn mono_preference_label(preference: &str) -> String {
    matched_option(preference)
        .map(|option| mono_option_title(option).to_owned())
        .unwrap_or_else(|| {
            if preference.is_empty() {
                "System default".to_owned()
            } else {
                preference.to_owned()
            }
        })
}

fn resolve_preference(preference: &str) -> String {
    let Some(option) = matched_option(preference) else {
        return pick(&[preference]).unwrap_or_else(|| default_mono().to_owned());
    };
    if option.id == SYSTEM_MONO_FONT_ID {
        return default_mono().to_owned();
    }
    pick(option.font_names).unwrap_or_else(|| default_mono().to_owned())
}

fn matched_option(preference: &str) -> Option<&'static MonoFontOption> {
    let preference = preference.trim();
    if preference.is_empty() {
        return mono_font_option(SYSTEM_MONO_FONT_ID);
    }
    mono_font_option_named(preference)
}

fn mono_option_title(option: &MonoFontOption) -> &str {
    if option.id == SYSTEM_MONO_FONT_ID {
        "System default"
    } else {
        option.title
    }
}

fn option_is_available(option: &MonoFontOption, source: &SystemSource) -> bool {
    option.id == SYSTEM_MONO_FONT_ID
        || option
            .font_names
            .iter()
            .any(|name| source.select_family_by_name(name).is_ok())
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
    fn empty_preference_uses_system_option() {
        assert_eq!(mono_preference_id(""), SYSTEM_MONO_FONT_ID);
        assert_eq!(mono_preference_label(""), "System default");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn system_default_uses_appkit_monospace_token_on_macos() {
        assert_eq!(resolve_preference(""), ".AppleSystemUIFontMonospaced");
    }

    #[test]
    fn literal_family_names_match_core_options() {
        assert_eq!(mono_preference_id("JetBrains Mono"), "jetbrains-mono");
        assert_eq!(
            mono_preference_id("JetBrainsMono-Regular"),
            "jetbrains-mono"
        );
        assert_eq!(
            mono_preference_label("JetBrainsMono-Regular"),
            "JetBrains Mono"
        );
    }
}
