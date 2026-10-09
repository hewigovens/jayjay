use std::collections::BTreeMap;
use std::path::Path;

mod platform;
mod specimen;

pub use specimen::FontSpecimen;

struct CodingFont {
    family: &'static str,
    /// Ids older versions saved for this family.
    legacy_ids: &'static [&'static str],
}

/// Fonts made for code, in default-preference order; the code-font picker lists the installed ones first.
const CODING_FONTS: &[CodingFont] = &[
    CodingFont {
        family: "JetBrains Mono",
        legacy_ids: &["jetbrains-mono"],
    },
    CodingFont {
        family: "Fira Code",
        legacy_ids: &["fira-code"],
    },
    CodingFont {
        family: "Cascadia Code",
        legacy_ids: &["cascadia-code"],
    },
    CodingFont {
        family: "Source Code Pro",
        legacy_ids: &["source-code-pro"],
    },
    CodingFont {
        family: "Google Sans Mono",
        legacy_ids: &["google-sans-mono"],
    },
    CodingFont {
        family: "Hack",
        legacy_ids: &["hack"],
    },
    CodingFont {
        family: "Ioskeley Mono",
        legacy_ids: &["ioskeley-mono"],
    },
    CodingFont {
        family: "IoskeleyMonoNL Nerd Font",
        legacy_ids: &["ioskeley-mono-nl-nerd-font", "ioskeleymono-nl-nerd-font"],
    },
    CodingFont {
        family: "Roboto Mono",
        legacy_ids: &["roboto-mono"],
    },
    CodingFont {
        family: "Inconsolata",
        legacy_ids: &["inconsolata"],
    },
    CodingFont {
        family: "Ubuntu Mono",
        legacy_ids: &["ubuntu-mono"],
    },
    CodingFont {
        family: "Menlo",
        legacy_ids: &["menlo"],
    },
    CodingFont {
        family: "SF Mono",
        legacy_ids: &["sf-mono"],
    },
    CodingFont {
        family: "Monaco",
        legacy_ids: &["monaco"],
    },
    CodingFont {
        family: "Berkeley Mono",
        legacy_ids: &["berkeley-mono"],
    },
    CodingFont {
        family: "Geist Mono",
        legacy_ids: &[],
    },
    CodingFont {
        family: "Paper Mono",
        legacy_ids: &[],
    },
];

pub fn coding_font_families() -> impl Iterator<Item = &'static str> {
    CODING_FONTS.iter().map(|font| font.family)
}

/// The family a saved code-font preference names; empty means the system monospace font.
pub fn mono_font_family(preference: &str) -> String {
    let preference = preference.trim();
    if preference == "system" {
        return String::new();
    }
    CODING_FONTS
        .iter()
        .find(|font| font.legacy_ids.contains(&preference))
        .map_or(preference, |font| font.family)
        .to_owned()
}

/// Installed monospaced families split into fonts made for code and the rest, each in picker order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeFontChoices {
    pub coding: Vec<String>,
    pub other: Vec<String>,
}

impl CodeFontChoices {
    pub fn new(monospaced_families: impl IntoIterator<Item = String>) -> Self {
        let (coding, other) = font_family_choices(monospaced_families)
            .into_iter()
            .partition(|family| is_coding_font(family));
        Self { coding, other }
    }
}

/// Nerd Fonts only patch programming fonts, and rename them (JetBrains Mono becomes "JetBrainsMono Nerd Font").
fn is_coding_font(family: &str) -> bool {
    family.contains("Nerd Font")
        || coding_font_families().any(|coding| coding.eq_ignore_ascii_case(family))
}

pub fn is_user_font_path(path: &Path) -> bool {
    platform::user_font_dirs()
        .iter()
        .any(|dir| path.starts_with(dir))
}

/// Picker order for installed families: deduped, case-insensitively sorted, without hidden system families.
pub fn font_family_choices(families: impl IntoIterator<Item = String>) -> Vec<String> {
    families
        .into_iter()
        .filter(|family| !family.trim().is_empty() && !family.starts_with('.'))
        .map(|family| (family.to_lowercase(), family))
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_ids_map_to_their_family_and_names_pass_through() {
        assert_eq!(mono_font_family("system"), "");
        assert_eq!(mono_font_family(""), "");
        assert_eq!(mono_font_family("jetbrains-mono"), "JetBrains Mono");
        assert_eq!(
            mono_font_family("ioskeleymono-nl-nerd-font"),
            "IoskeleyMonoNL Nerd Font"
        );
        assert_eq!(mono_font_family(" Paper Mono "), "Paper Mono");
    }

    #[test]
    fn code_font_choices_put_coding_fonts_first() {
        let choices = CodeFontChoices::new(
            [
                "SimSong",
                "Menlo",
                "BIZ UDGothic",
                "fira code",
                "Paper Mono",
                "JetBrainsMono Nerd Font Mono",
            ]
            .map(str::to_owned),
        );
        assert_eq!(
            choices.coding,
            [
                "fira code",
                "JetBrainsMono Nerd Font Mono",
                "Menlo",
                "Paper Mono"
            ]
        );
        assert_eq!(choices.other, ["BIZ UDGothic", "SimSong"]);
    }

    #[test]
    fn family_choices_are_sorted_deduped_and_skip_hidden_families() {
        let families = font_family_choices(
            ["paper Mono", "Menlo", ".SF NS Mono", "Ab Mono", "Ab Mono"].map(str::to_owned),
        );
        assert_eq!(families, ["Ab Mono", "Menlo", "paper Mono"]);
    }

    #[test]
    fn user_font_dirs_are_user_fonts() {
        for dir in platform::user_font_dirs() {
            assert!(is_user_font_path(&dir.join("Paper Mono.otf")));
        }
        #[cfg(target_os = "macos")]
        assert!(!is_user_font_path(Path::new(
            "/System/Library/Fonts/Menlo.ttc"
        )));
        #[cfg(target_os = "linux")]
        assert!(!is_user_font_path(Path::new(
            "/usr/share/fonts/TTF/DejaVuSansMono.ttf"
        )));
    }
}
