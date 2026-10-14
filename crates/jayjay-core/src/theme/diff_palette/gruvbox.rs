use super::DiffPalette;

pub(super) fn light() -> DiffPalette {
    DiffPalette::themed(
        "gruvbox-light",
        "Gruvbox Light",
        0xfbf1c7,
        0x3c3836,
        [
            0x9d0006, 0x79740e, 0xb57614, 0x076678, 0x8f3f71, 0x427b58, 0xaf3a03,
        ],
    )
}

pub(super) fn dark() -> DiffPalette {
    DiffPalette::themed(
        "gruvbox-dark",
        "Gruvbox Dark",
        0x282828,
        0xebdbb2,
        [
            0xfb4934, 0xb8bb26, 0xfabd2f, 0x83a598, 0xd3869b, 0x8ec07c, 0xfe8019,
        ],
    )
}
