use super::super::{
    BERKELEY_MONO, CASCADIA_CODE, FIRA_CODE, GOOGLE_SANS_MONO, HACK, INCONSOLATA, IOSKELEY_MONO,
    IOSKELEY_MONO_NL_NERD_FONT, JETBRAINS_MONO, MENLO, MONACO, MonoFontOption, ROBOTO_MONO,
    SF_MONO, SOURCE_CODE_PRO, SYSTEM_MONO,
};

pub const MONO_FONT_OPTIONS: &[MonoFontOption] = &[
    SYSTEM_MONO,
    MENLO,
    SF_MONO,
    JETBRAINS_MONO,
    FIRA_CODE,
    CASCADIA_CODE,
    SOURCE_CODE_PRO,
    GOOGLE_SANS_MONO,
    HACK,
    IOSKELEY_MONO,
    IOSKELEY_MONO_NL_NERD_FONT,
    BERKELEY_MONO,
    ROBOTO_MONO,
    INCONSOLATA,
    MONACO,
];

pub const MONO_FONT_FALLBACK_NAMES: &[&str] = &[
    "SF Mono",
    "JetBrains Mono",
    "Fira Code",
    "Cascadia Code",
    "Source Code Pro",
    "Google Sans Mono",
    "Hack",
    "Ioskeley Mono",
    "IoskeleyMonoNL Nerd Font",
    "Berkeley Mono",
    "Roboto Mono",
    "Inconsolata",
    "Menlo",
    "Monaco",
];
