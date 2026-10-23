use super::DiffPalette;
use crate::theme::DiffThemeColors;

pub(super) fn light() -> DiffPalette {
    DiffPalette::default_for(false, DiffThemeColors::light())
}

pub(super) fn dark() -> DiffPalette {
    DiffPalette::default_for(true, DiffThemeColors::dark())
}
