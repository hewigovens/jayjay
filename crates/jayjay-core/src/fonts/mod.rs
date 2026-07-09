mod platform;

pub use platform::{MONO_FONT_FALLBACK_NAMES, MONO_FONT_OPTIONS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonoFontOption {
    pub id: &'static str,
    pub title: &'static str,
    pub font_names: &'static [&'static str],
}

pub const SYSTEM_MONO_FONT_ID: &str = "system";

pub fn mono_font_option(id: &str) -> Option<&'static MonoFontOption> {
    MONO_FONT_OPTIONS.iter().find(|option| option.id == id)
}

pub(crate) const SYSTEM_MONO: MonoFontOption = MonoFontOption {
    id: SYSTEM_MONO_FONT_ID,
    title: "System Mono",
    font_names: &[],
};

pub(crate) const JETBRAINS_MONO: MonoFontOption = MonoFontOption {
    id: "jetbrains-mono",
    title: "JetBrains Mono",
    font_names: &["JetBrains Mono", "JetBrainsMono-Regular"],
};

pub(crate) const FIRA_CODE: MonoFontOption = MonoFontOption {
    id: "fira-code",
    title: "Fira Code",
    font_names: &["Fira Code", "FiraCode-Regular"],
};

pub(crate) const CASCADIA_CODE: MonoFontOption = MonoFontOption {
    id: "cascadia-code",
    title: "Cascadia Code",
    font_names: &["Cascadia Code", "CascadiaCode-Regular"],
};

pub(crate) const SOURCE_CODE_PRO: MonoFontOption = MonoFontOption {
    id: "source-code-pro",
    title: "Source Code Pro",
    font_names: &["Source Code Pro", "SourceCodePro-Regular"],
};

pub(crate) const GOOGLE_SANS_MONO: MonoFontOption = MonoFontOption {
    id: "google-sans-mono",
    title: "Google Sans Mono",
    font_names: &[
        "Google Sans Mono",
        "Google-Sans-Mono",
        "GoogleSansMono-Regular",
    ],
};

pub(crate) const HACK: MonoFontOption = MonoFontOption {
    id: "hack",
    title: "Hack",
    font_names: &["Hack", "Hack-Regular"],
};

pub(crate) const IOSKELEY_MONO: MonoFontOption = MonoFontOption {
    id: "ioskeley-mono",
    title: "Ioskeley Mono",
    font_names: &["Ioskeley Mono", "Ioskeley-Mono"],
};

pub(crate) const IOSKELEY_MONO_NL_NERD_FONT: MonoFontOption = MonoFontOption {
    id: "ioskeley-mono-nl-nerd-font",
    title: "IoskeleyMonoNL Nerd Font",
    font_names: &["IoskeleyMonoNL Nerd Font", "IoskeleyMonoNLNF"],
};

pub(crate) const ROBOTO_MONO: MonoFontOption = MonoFontOption {
    id: "roboto-mono",
    title: "Roboto Mono",
    font_names: &["Roboto Mono", "RobotoMono-Regular"],
};

pub(crate) const INCONSOLATA: MonoFontOption = MonoFontOption {
    id: "inconsolata",
    title: "Inconsolata",
    font_names: &["Inconsolata"],
};

#[cfg(not(target_os = "macos"))]
pub(crate) const UBUNTU_MONO: MonoFontOption = MonoFontOption {
    id: "ubuntu-mono",
    title: "Ubuntu Mono",
    font_names: &["Ubuntu Mono", "UbuntuMono-Regular"],
};

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn mono_font_options_have_unique_ids_and_titles() {
        let mut ids = HashSet::new();
        let mut titles = HashSet::new();
        for option in MONO_FONT_OPTIONS {
            assert!(!option.id.is_empty());
            assert!(!option.title.is_empty());
            assert!(ids.insert(option.id), "duplicate font id {}", option.id);
            assert!(
                titles.insert(option.title),
                "duplicate font title {}",
                option.title
            );
        }
    }

    #[test]
    fn system_font_option_is_first() {
        assert_eq!(
            MONO_FONT_OPTIONS.first().map(|option| option.id),
            Some(SYSTEM_MONO_FONT_ID)
        );
    }

    #[test]
    fn fallback_names_are_declared_by_options() {
        for fallback_name in MONO_FONT_FALLBACK_NAMES {
            assert!(
                MONO_FONT_OPTIONS
                    .iter()
                    .any(|option| option.font_names.contains(fallback_name)),
                "fallback font {fallback_name} is missing from MONO_FONT_OPTIONS"
            );
        }
    }
}
