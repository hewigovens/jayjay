use gpui::WindowAppearance;

use super::Theme;
use crate::app::config::AppearanceMode;

#[test]
fn system_appearance_follows_window_appearance() {
    assert!(!Theme::for_appearance(AppearanceMode::System, WindowAppearance::Light).is_dark);
    assert!(Theme::for_appearance(AppearanceMode::System, WindowAppearance::Dark).is_dark);
}

#[test]
fn explicit_appearance_overrides_window_appearance() {
    assert!(!Theme::for_appearance(AppearanceMode::Light, WindowAppearance::Dark).is_dark);
    assert!(Theme::for_appearance(AppearanceMode::Dark, WindowAppearance::Light).is_dark);
}

#[test]
fn builtin_themes_keep_readable_secondary_text() {
    for theme in [
        Theme::light(),
        Theme::for_appearance(AppearanceMode::Dark, WindowAppearance::Light),
    ] {
        let bg = jayjay_core::theme::luminance(theme.sidebar_bg);
        for text in [theme.fg, theme.fg_dim, theme.tag_fg, theme.toggle_active_fg] {
            assert!(
                (jayjay_core::theme::luminance(text) - bg).abs() > 0.25,
                "{text:06x}"
            );
        }
        assert_eq!(theme.diff_context_bg, theme.detail_bg);
    }
}
