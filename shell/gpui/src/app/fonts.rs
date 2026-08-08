use std::sync::{OnceLock, RwLock};

use font_kit::source::SystemSource;
use gpui::{App, Pixels, font, px};
use jayjay_core::{MONO_FONT_OPTIONS, MonoFontOption, SYSTEM_MONO_FONT_ID, mono_font_option};

use crate::app::config::AppConfig;

mod platform;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonoFontChoice {
    pub(crate) id: String,
    pub(crate) title: String,
}

static MONO: OnceLock<RwLock<String>> = OnceLock::new();

pub(crate) fn mono() -> String {
    MONO.get_or_init(|| RwLock::new(String::new()))
        .read()
        .map(|font| {
            if font.is_empty() {
                platform::platform_default_mono()
            } else {
                font.clone()
            }
        })
        .unwrap_or_else(|_| platform::platform_default_mono())
}

// Falls back to ~7.2 px (SF Mono / Menlo at 12 px) on measurement failure.
pub fn mono_advance(cx: &App, size: Pixels) -> Pixels {
    let font_id = cx.text_system().resolve_font(&font(mono()));
    cx.text_system()
        .ch_advance(font_id, size)
        .unwrap_or(px(7.2))
}

pub(crate) fn sync_from_config(config: &AppConfig) {
    let resolved = resolve_preference(&config.font_family);
    if let Ok(mut font) = MONO.get_or_init(|| RwLock::new(String::new())).write() {
        *font = resolved;
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
        let fallback = platform::platform_default_mono();
        return pick(&[preference], &fallback);
    };
    if option.id == SYSTEM_MONO_FONT_ID {
        return platform::platform_default_mono();
    }
    let fallback = platform::platform_default_mono();
    pick(option.font_names, &fallback)
}

fn matched_option(preference: &str) -> Option<&'static MonoFontOption> {
    let preference = preference.trim();
    if preference.is_empty() {
        return mono_font_option(SYSTEM_MONO_FONT_ID);
    }
    mono_font_option(preference).or_else(|| {
        MONO_FONT_OPTIONS
            .iter()
            .find(|option| option.title == preference || option.font_names.contains(&preference))
    })
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

fn pick(candidates: &[&str], fallback: &str) -> String {
    let source = SystemSource::new();
    for name in candidates {
        if source.select_family_by_name(name).is_ok() {
            return (*name).to_owned();
        }
    }
    fallback.to_owned()
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
