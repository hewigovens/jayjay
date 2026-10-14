mod color;
mod diff_colors;
mod diff_palette;
mod repo_colors;
mod seed;

pub use color::{contrast_ratio, is_dark, luminance, mix};
pub use diff_colors::DiffThemeColors;
pub use diff_palette::{DiffPalette, diff_palettes};
pub use repo_colors::{RepoColors, repo_colors};
pub use seed::ThemeSeed;

#[cfg(test)]
mod tests;
