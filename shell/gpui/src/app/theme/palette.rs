use gpui::WindowAppearance;
use jayjay_core::{DiffThemeColors, RepoColors, ThemeSeed};

use super::Theme;
use crate::app::config::{AppearanceMode, ThemeSelection};

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
        palette: &ThemeSelection,
    ) -> Self {
        let is_dark = match mode {
            AppearanceMode::Light => false,
            AppearanceMode::Dark => true,
            AppearanceMode::System => matches!(
                system,
                WindowAppearance::Dark | WindowAppearance::VibrantDark
            ),
        };
        let resolved = palette.resolve(is_dark);
        Self::from_seed(&resolved.seed, resolved.colors, resolved.repo)
    }
}
