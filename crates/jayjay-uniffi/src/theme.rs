#[uniffi::export]
pub fn diff_theme_colors(is_dark: bool) -> jayjay_core::DiffThemeColors {
    jayjay_core::diff_theme_colors(is_dark)
}
