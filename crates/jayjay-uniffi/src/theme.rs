#[uniffi::export]
fn color_themes() -> Vec<jayjay_core::ColorTheme> {
    jayjay_core::color_themes()
}

#[uniffi::export]
fn color_theme(id: String, is_dark: bool) -> jayjay_core::ColorTheme {
    jayjay_core::ColorTheme::resolve(&id, is_dark)
}

#[uniffi::export]
fn default_color_theme_id(is_dark: bool) -> String {
    jayjay_core::ColorTheme::default_id(is_dark).to_owned()
}
