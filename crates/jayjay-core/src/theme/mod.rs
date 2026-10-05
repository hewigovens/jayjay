mod color;
mod diff_colors;
mod repo_colors;
mod seed;

pub use color::{is_dark, luminance, mix};
pub use diff_colors::{DiffThemeColors, diff_theme_colors};
pub use repo_colors::{RepoColors, repo_colors};
pub use seed::ThemeSeed;

#[cfg(test)]
mod tests;
