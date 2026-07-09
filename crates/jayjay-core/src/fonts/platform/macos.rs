use super::super::{
    CASCADIA_CODE, FIRA_CODE, GOOGLE_SANS_MONO, HACK, INCONSOLATA, IOSKELEY_MONO,
    IOSKELEY_MONO_NL_NERD_FONT, JETBRAINS_MONO, MonoFontOption, ROBOTO_MONO, SOURCE_CODE_PRO,
    SYSTEM_MONO,
};

const MENLO: MonoFontOption = MonoFontOption {
    id: "menlo",
    title: "Menlo",
    font_names: &["Menlo"],
};

const SF_MONO: MonoFontOption = MonoFontOption {
    id: "sf-mono",
    title: "SF Mono",
    font_names: &["SF Mono"],
};

const BERKELEY_MONO: MonoFontOption = MonoFontOption {
    id: "berkeley-mono",
    title: "Berkeley Mono",
    font_names: &["Berkeley Mono", "BerkeleyMono-Regular"],
};

const MONACO: MonoFontOption = MonoFontOption {
    id: "monaco",
    title: "Monaco",
    font_names: &["Monaco"],
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
