mod color;
mod color_theme;
mod diff_colors;
mod repo_colors;
mod seed;

pub use color::{contrast_ratio, is_dark, luminance, mix};
pub use color_theme::{ColorTheme, color_themes};
pub use diff_colors::DiffThemeColors;
pub use repo_colors::RepoColors;
pub use seed::ThemeSeed;

#[cfg(test)]
mod tests;
