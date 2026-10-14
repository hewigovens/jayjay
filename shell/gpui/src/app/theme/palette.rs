use gpui::WindowAppearance;
use jayjay_core::{DiffThemeColors, RepoColors, ThemeSeed};

use super::Theme;
use crate::app::config::{AppearanceMode, DiffPaletteSelection};

impl Theme {
    pub fn light() -> Self {
        Self::from_seed(
            &ThemeSeed::light(),
            DiffThemeColors::light(),
            RepoColors::light(),
        )
    }

    pub fn for_appearance(
        mode: AppearanceMode,
        system: WindowAppearance,
        palette: &DiffPaletteSelection,
    ) -> Self {
        let is_dark = match mode {
            AppearanceMode::Light => false,
            AppearanceMode::Dark => true,
            AppearanceMode::System => matches!(
                system,
                WindowAppearance::Dark | WindowAppearance::VibrantDark
            ),
        };
        let (seed, repo) = if is_dark {
            (ThemeSeed::dark(), RepoColors::dark())
        } else {
            (ThemeSeed::light(), RepoColors::light())
        };
        Self::from_seed(&seed, palette.resolve(is_dark).colors, repo)
    }
}
