mod color;
mod diff_colors;
mod seed;

pub use color::{is_dark, luminance, mix};
pub use diff_colors::{DiffThemeColors, change_id_prefix_color, diff_theme_colors};
pub use seed::ThemeSeed;

#[cfg(test)]
mod tests;
