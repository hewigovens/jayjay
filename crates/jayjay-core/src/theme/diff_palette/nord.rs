use super::DiffPalette;

pub(super) fn dark() -> DiffPalette {
    DiffPalette::themed(
        "nord",
        "Nord",
        0x2e3440,
        0xd8dee9,
        [
            0xbf616a, 0xa3be8c, 0xebcb8b, 0x81a1c1, 0xb48ead, 0x88c0d0, 0xd08770,
        ],
    )
}
