#[uniffi::export]
fn diff_palettes() -> Vec<jayjay_core::DiffPalette> {
    jayjay_core::diff_palettes()
}

#[uniffi::export]
fn diff_palette(id: String, is_dark: bool) -> jayjay_core::DiffPalette {
    jayjay_core::DiffPalette::resolve(&id, is_dark)
}

#[uniffi::export]
fn default_diff_palette_id(is_dark: bool) -> String {
    jayjay_core::DiffPalette::default_id(is_dark).to_owned()
}

#[uniffi::export]
fn repo_colors(is_dark: bool) -> jayjay_core::RepoColors {
    jayjay_core::repo_colors(is_dark)
}
