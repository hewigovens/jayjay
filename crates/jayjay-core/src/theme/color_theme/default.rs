use super::ColorTheme;
use crate::theme::{DiffThemeColors, RepoColors, ThemeSeed};

pub(super) fn light() -> ColorTheme {
    ColorTheme::default_for(
        false,
        ThemeSeed::light(),
        DiffThemeColors::light(),
        RepoColors::light(),
    )
}

pub(super) fn dark() -> ColorTheme {
    ColorTheme::default_for(
        true,
        ThemeSeed::dark(),
        DiffThemeColors::dark(),
        RepoColors::dark(),
    )
}
