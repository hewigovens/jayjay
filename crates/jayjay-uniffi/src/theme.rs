#[uniffi::export]
fn diff_theme_colors(is_dark: bool) -> jayjay_core::DiffThemeColors {
    jayjay_core::diff_theme_colors(is_dark)
}

#[uniffi::export]
fn repo_colors(is_dark: bool) -> jayjay_core::RepoColors {
    jayjay_core::repo_colors(is_dark)
}
