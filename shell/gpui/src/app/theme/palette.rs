use gpui::WindowAppearance;
use jayjay_core::{DiffThemeColors, RepoColors, ThemeSeed};

use super::Theme;
use crate::app::config::AppearanceMode;

impl Theme {
    pub fn light() -> Self {
        Self::from_seed(
            &ThemeSeed::light(),
            DiffThemeColors::light(),
            RepoColors::light(),
        )
    }

    fn dark() -> Self {
        Self::from_seed(
            &ThemeSeed::dark(),
            DiffThemeColors::dark(),
            RepoColors::dark(),
        )
    }

    pub fn for_appearance(mode: AppearanceMode, system: WindowAppearance) -> Self {
        match mode {
            AppearanceMode::Light => Self::light(),
            AppearanceMode::Dark => Self::dark(),
            AppearanceMode::System => match system {
                WindowAppearance::Light | WindowAppearance::VibrantLight => Self::light(),
                WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::dark(),
            },
        }
    }
}
