use super::ColorTheme;

pub(super) fn light() -> ColorTheme {
    ColorTheme::themed(
        "solarized-light",
        "Solarized Light",
        0xfdf6e3,
        0x586e75,
        ACCENTS,
    )
}

pub(super) fn dark() -> ColorTheme {
    ColorTheme::themed(
        "solarized-dark",
        "Solarized Dark",
        0x002b36,
        0x93a1a1,
        ACCENTS,
    )
}

const ACCENTS: [u32; 7] = [
    0xdc322f, 0x859900, 0xb58900, 0x268bd2, 0xd33682, 0x2aa198, 0xcb4b16,
];
