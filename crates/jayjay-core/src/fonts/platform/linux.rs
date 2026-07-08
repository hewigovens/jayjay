use super::super::{
    CASCADIA_CODE, FIRA_CODE, GOOGLE_SANS_MONO, HACK, INCONSOLATA, IOSKELEY_MONO,
    IOSKELEY_MONO_NL_NERD_FONT, JETBRAINS_MONO, MonoFontOption, ROBOTO_MONO, SOURCE_CODE_PRO,
    SYSTEM_MONO, UBUNTU_MONO,
};

pub const MONO_FONT_OPTIONS: &[MonoFontOption] = &[
    SYSTEM_MONO,
    JETBRAINS_MONO,
    FIRA_CODE,
    CASCADIA_CODE,
    SOURCE_CODE_PRO,
    GOOGLE_SANS_MONO,
    HACK,
    IOSKELEY_MONO,
    IOSKELEY_MONO_NL_NERD_FONT,
    ROBOTO_MONO,
    INCONSOLATA,
    UBUNTU_MONO,
];

pub const MONO_FONT_FALLBACK_NAMES: &[&str] = &[
    "JetBrains Mono",
    "Fira Code",
    "Cascadia Code",
    "Source Code Pro",
    "Google Sans Mono",
    "Hack",
    "Ioskeley Mono",
    "IoskeleyMonoNL Nerd Font",
    "Roboto Mono",
    "Inconsolata",
    "Ubuntu Mono",
];
