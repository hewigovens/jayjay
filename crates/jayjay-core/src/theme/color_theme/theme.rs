use super::catalog::CATALOG;
use crate::theme::{DiffThemeColors, RepoColors, ThemeSeed};

const DEFAULT_LIGHT_COLOR_THEME_ID: &str = "jayjay-light";
const DEFAULT_DARK_COLOR_THEME_ID: &str = "jayjay-dark";

#[derive(Debug, Clone, PartialEq)]
pub struct ColorTheme {
    pub id: String,
    pub name: String,
    pub is_dark: bool,
    pub seed: ThemeSeed,
    pub colors: DiffThemeColors,
    pub repo: RepoColors,
}

impl ColorTheme {
    pub(super) fn new(
        id: &str,
        name: &str,
        seed: ThemeSeed,
        colors: DiffThemeColors,
        repo: RepoColors,
    ) -> Self {
        Self {
            id: id.to_owned(),
            name: name.to_owned(),
            is_dark: seed.is_dark(),
            seed,
            colors,
            repo,
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
        let colors = DiffThemeColors::from_seed(&seed);
        let repo = RepoColors::from_seed(&seed);
        Self::new(id, name, seed, colors, repo)
    }

    pub(super) fn default_for(
        is_dark: bool,
        seed: ThemeSeed,
        colors: DiffThemeColors,
        repo: RepoColors,
    ) -> Self {
        Self::new(Self::default_id(is_dark), "", seed, colors, repo)
    }

    pub fn default_id(is_dark: bool) -> &'static str {
        if is_dark {
            DEFAULT_DARK_COLOR_THEME_ID
        } else {
            DEFAULT_LIGHT_COLOR_THEME_ID
        }
    }

    /// An unknown id, or one from the other side, falls back to that side's default.
    pub fn resolve(id: &str, is_dark: bool) -> Self {
        let find = |id: &str| CATALOG.iter().find(|t| t.is_dark == is_dark && t.id == id);
        find(id)
            .or_else(|| find(Self::default_id(is_dark)))
            .expect("the catalog holds both defaults")
            .localized()
    }

    pub(super) fn localized(&self) -> Self {
        let name = match self.id.as_str() {
            DEFAULT_LIGHT_COLOR_THEME_ID => jayjay_i18n::theme::jayjay_light(),
            DEFAULT_DARK_COLOR_THEME_ID => jayjay_i18n::theme::jayjay_dark(),
            _ => return self.clone(),
        };
        Self {
            name,
            ..self.clone()
        }
    }
}
