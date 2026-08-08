#[uniffi::export]
fn diff_theme_colors(is_dark: bool) -> jayjay_core::DiffThemeColors {
    jayjay_core::diff_theme_colors(is_dark)
}

/// Change/commit-id prefix highlight color (`0xRRGGBB`) for the given appearance.
#[uniffi::export]
fn change_id_prefix_color(is_dark: bool) -> u32 {
    jayjay_core::change_id_prefix_color(is_dark)
}
