use super::DiffPalette;

pub(super) fn light() -> DiffPalette {
    DiffPalette::themed(
        "one-light",
        "One Light",
        0xfafafa,
        0x383a42,
        [
            0xe45649, 0x50a14f, 0xc18401, 0x4078f2, 0xa626a4, 0x0184bc, 0x986801,
        ],
    )
}

pub(super) fn dark() -> DiffPalette {
    DiffPalette::themed(
        "one-dark",
        "One Dark",
        0x282c34,
        0xabb2bf,
        [
            0xe06c75, 0x98c379, 0xe5c07b, 0x61afef, 0xc678dd, 0x56b6c2, 0xd19a66,
        ],
    )
}
