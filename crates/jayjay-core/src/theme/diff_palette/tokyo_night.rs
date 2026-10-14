use super::DiffPalette;

pub(super) fn dark() -> DiffPalette {
    DiffPalette::themed(
        "tokyo-night",
        "Tokyo Night",
        0x1a1b26,
        0xc0caf5,
        [
            0xf7768e, 0x9ece6a, 0xe0af68, 0x7aa2f7, 0xbb9af7, 0x7dcfff, 0xff9e64,
        ],
    )
}
