use super::DiffPalette;
use crate::theme::DiffThemeColors;

pub(super) fn light() -> DiffPalette {
    DiffPalette::new(
        DiffPalette::default_id(false),
        "Default Light",
        false,
        DiffThemeColors::light(),
    )
}

pub(super) fn dark() -> DiffPalette {
    DiffPalette::new(
        DiffPalette::default_id(true),
        "Default Dark",
        true,
        DiffThemeColors::dark(),
    )
}
