use super::DiffPalette;

pub(super) fn light() -> DiffPalette {
    DiffPalette::themed(
        "catppuccin-latte",
        "Catppuccin Latte",
        0xeff1f5,
        0x4c4f69,
        [
            0xd20f39, 0x40a02b, 0xdf8e1d, 0x1e66f5, 0x8839ef, 0x179299, 0xfe640b,
        ],
    )
}

pub(super) fn dark() -> DiffPalette {
    DiffPalette::themed(
        "catppuccin-mocha",
        "Catppuccin Mocha",
        0x1e1e2e,
        0xcdd6f4,
        [
            0xf38ba8, 0xa6e3a1, 0xf9e2af, 0x89b4fa, 0xcba6f7, 0x94e2d5, 0xfab387,
        ],
    )
}
