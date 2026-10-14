use gpui::WindowAppearance;

use super::Theme;
use crate::app::config::{AppearanceMode, DiffPaletteSelection};

#[test]
fn system_appearance_follows_window_appearance() {
    assert!(
        !Theme::for_appearance(
            AppearanceMode::System,
            WindowAppearance::Light,
            &DiffPaletteSelection::default()
        )
        .is_dark
    );
    assert!(
        Theme::for_appearance(
            AppearanceMode::System,
            WindowAppearance::Dark,
            &DiffPaletteSelection::default()
        )
        .is_dark
    );
}

#[test]
fn explicit_appearance_overrides_window_appearance() {
    assert!(
        !Theme::for_appearance(
            AppearanceMode::Light,
            WindowAppearance::Dark,
            &DiffPaletteSelection::default()
        )
        .is_dark
    );
    assert!(
        Theme::for_appearance(
            AppearanceMode::Dark,
            WindowAppearance::Light,
            &DiffPaletteSelection::default()
        )
        .is_dark
    );
}

#[test]
fn builtin_themes_keep_readable_secondary_text() {
    for theme in [
        Theme::light(),
        Theme::for_appearance(
            AppearanceMode::Dark,
            WindowAppearance::Light,
            &DiffPaletteSelection::default(),
        ),
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

#[test]
fn code_line_height_scales_with_code_size_and_line_spacing() {
    let config = crate::app::config::AppConfig {
        code_font_size: 13.,
        code_line_spacing: 1.5,
        ..Default::default()
    };
    let theme = Theme::light().with_typography(&config);
    assert_eq!(theme.code_line_height(), 27.);
    assert_eq!(theme.scaled_font_size(12.), 12.);
}

#[test]
fn a_selected_diff_palette_recolors_diff_syntax_but_not_chrome() {
    let nord = DiffPaletteSelection {
        dark: "nord".to_owned(),
        ..DiffPaletteSelection::default()
    };
    let default = Theme::for_appearance(
        AppearanceMode::Dark,
        WindowAppearance::Dark,
        &DiffPaletteSelection::default(),
    );
    let themed = Theme::for_appearance(AppearanceMode::Dark, WindowAppearance::Dark, &nord);

    assert_ne!(themed.diff_syntax, default.diff_syntax);
    assert_eq!(themed.syntax, default.syntax);
    assert_eq!(themed.success_fg, default.success_fg);
    assert_eq!(
        (themed.added_fg, themed.removed_fg),
        (default.added_fg, default.removed_fg)
    );
    assert_ne!(themed.diff_gutter_added_fg, default.diff_gutter_added_fg);
}
