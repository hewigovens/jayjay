use gpui::WindowAppearance;

use super::Theme;
use crate::app::config::{AppearanceMode, ThemeSelection};

#[test]
fn system_appearance_follows_window_appearance() {
    assert!(
        !Theme::for_appearance(
            AppearanceMode::System,
            WindowAppearance::Light,
            &ThemeSelection::default()
        )
        .is_dark
    );
    assert!(
        Theme::for_appearance(
            AppearanceMode::System,
            WindowAppearance::Dark,
            &ThemeSelection::default()
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
            &ThemeSelection::default()
        )
        .is_dark
    );
    assert!(
        Theme::for_appearance(
            AppearanceMode::Dark,
            WindowAppearance::Light,
            &ThemeSelection::default()
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
            &ThemeSelection::default(),
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
fn a_selected_theme_recolors_chrome_repo_and_diff_surfaces_while_preserving_fonts() {
    let nord = ThemeSelection {
        dark: "nord".to_owned(),
        ..ThemeSelection::default()
    };
    let default = Theme::for_appearance(
        AppearanceMode::Dark,
        WindowAppearance::Dark,
        &ThemeSelection::default(),
    );
    let themed = Theme::for_appearance(AppearanceMode::Dark, WindowAppearance::Dark, &nord);

    assert_ne!(themed.detail_bg, default.detail_bg);
    assert_ne!(themed.sidebar_bg, default.sidebar_bg);
    assert_ne!(themed.fg, default.fg);
    assert_ne!(themed.selected_bg, default.selected_bg);
    assert_ne!(themed.dag_line, default.dag_line);
    assert_ne!(themed.change_id_prefix, default.change_id_prefix);
    assert_ne!(themed.file_added_color, default.file_added_color);
    assert_ne!(themed.file_removed_color, default.file_removed_color);
    assert_ne!(themed.file_modified_color, default.file_modified_color);
    assert_ne!(themed.file_renamed_color, default.file_renamed_color);
    assert_ne!(themed.file_lfs_color, default.file_lfs_color);
    assert_ne!(themed.note_accent, default.note_accent);
    assert_ne!(themed.syntax, default.syntax);
    assert_ne!(themed.success_fg, default.success_fg);
    assert_ne!(
        (themed.added_fg, themed.removed_fg),
        (default.added_fg, default.removed_fg)
    );
    assert_ne!(themed.diff_gutter_added_fg, default.diff_gutter_added_fg);
    assert_eq!(themed.font_size, default.font_size);
    assert_eq!(themed.code_font_size, default.code_font_size);
    assert_eq!(themed.code_line_spacing, default.code_line_spacing);
}
