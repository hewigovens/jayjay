use gpui::WindowAppearance;
use jayjay_core::{DiffThemeColors, ThemeSeed};

use super::Theme;
use crate::app::config::AppearanceMode;

impl Theme {
    pub fn light() -> Self {
        let mut theme = Self::from_seed(&ThemeSeed::light(), DiffThemeColors::light());
        theme.change_id_prefix = jayjay_core::change_id_prefix_color(false);
        theme
    }

    fn dark() -> Self {
        let mut theme = Self::from_seed(&ThemeSeed::dark(), DiffThemeColors::dark());
        theme.change_id_prefix = jayjay_core::change_id_prefix_color(true);
        theme
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
