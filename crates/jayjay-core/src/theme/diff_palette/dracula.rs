use super::DiffPalette;

pub(super) fn dark() -> DiffPalette {
    DiffPalette::themed(
        "dracula",
        "Dracula",
        0x282a36,
        0xf8f8f2,
        [
            0xff5555, 0x50fa7b, 0xf1fa8c, 0xbd93f9, 0xff79c6, 0x8be9fd, 0xffb86c,
        ],
    )
}
