use super::diff_palettes;
use crate::theme::{DiffThemeColors, ThemeSeed};

const DEFAULT_LIGHT_DIFF_PALETTE_ID: &str = "jayjay-light";
const DEFAULT_DARK_DIFF_PALETTE_ID: &str = "jayjay-dark";

#[derive(Debug, Clone, PartialEq)]
pub struct DiffPalette {
    pub id: String,
    pub name: String,
    pub is_dark: bool,
    pub colors: DiffThemeColors,
}

impl DiffPalette {
    pub(super) fn new(id: &str, name: &str, is_dark: bool, colors: DiffThemeColors) -> Self {
        Self {
            id: id.to_owned(),
            name: name.to_owned(),
            is_dark,
            colors,
        }
    }

    /// Accents are `[red, green, yellow, blue, magenta, cyan, orange]` from the theme's published palette.
    pub(super) fn themed(
        id: &str,
        name: &str,
        background: u32,
        foreground: u32,
        accents: [u32; 7],
    ) -> Self {
        let [red, green, yellow, blue, magenta, cyan, orange] = accents;
        let seed = ThemeSeed {
            red,
            green,
            yellow,
            blue,
            magenta,
            cyan,
            orange,
            ..ThemeSeed::from_pair(background, foreground)
        };
        Self::new(id, name, seed.is_dark(), DiffThemeColors::from_seed(&seed))
    }

    pub fn default_id(is_dark: bool) -> &'static str {
        if is_dark {
            DEFAULT_DARK_DIFF_PALETTE_ID
        } else {
            DEFAULT_LIGHT_DIFF_PALETTE_ID
        }
    }

    /// An unknown id, or one from the other side, falls back to that side's default.
    pub fn resolve(id: &str, is_dark: bool) -> Self {
        let palettes = diff_palettes();
        let find = |id: &str| palettes.iter().find(|p| p.is_dark == is_dark && p.id == id);
        find(id)
            .or_else(|| find(Self::default_id(is_dark)))
            .expect("the catalog holds both defaults")
            .clone()
    }
}
