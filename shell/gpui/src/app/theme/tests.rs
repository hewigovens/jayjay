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
