use super::*;

#[test]
fn mix_interpolates_per_channel_and_clamps() {
    assert_eq!(mix(0x000000, 0xffffff, 0.5), 0x808080);
    assert_eq!(mix(0xff0000, 0x0000ff, 0.), 0xff0000);
    assert_eq!(mix(0xff0000, 0x0000ff, 2.), 0x0000ff);
    assert_eq!(mix(0x204060, 0x406080, 0.5), 0x305070);
}

#[test]
fn luminance_separates_light_and_dark_backgrounds() {
    assert!(is_dark(0x10131a));
    assert!(is_dark(0x002b36));
    assert!(!is_dark(0xffffff));
    assert!(!is_dark(0xfdf6e3));
    assert!((luminance(0xffffff) - 1.).abs() < 1e-5);
    assert!((luminance(0x0a0a0a) - 10. / 255. / 12.92).abs() < 1e-6);
}

#[test]
fn derived_seed_fills_every_optional_token_from_the_pair() {
    let dark = ThemeSeed::from_pair(0x1a1b26, 0xc0caf5);
    assert!(dark.is_dark());
    assert_ne!(dark.surface, dark.background);
    assert_ne!(dark.selection, dark.background);
    assert_eq!(dark.red, ThemeSeed::dark().red);

    let light = ThemeSeed::from_pair(0xfdf6e3, 0x586e75);
    assert!(!light.is_dark());
    assert_eq!(light.green, ThemeSeed::light().green);
    assert!(luminance(light.muted) > luminance(light.foreground));
}

#[test]
fn default_color_theme_ids_resolve_to_hand_tuned_themes() {
    let light = ColorTheme::resolve(ColorTheme::default_id(false), false);
    assert_eq!(light.seed, ThemeSeed::light());
    assert_eq!(light.colors, DiffThemeColors::light());
    assert_eq!(light.repo, RepoColors::light());

    let dark = ColorTheme::resolve(ColorTheme::default_id(true), true);
    assert_eq!(dark.seed, ThemeSeed::dark());
    assert_eq!(dark.colors, DiffThemeColors::dark());
    assert_eq!(dark.repo, RepoColors::dark());
}

#[test]
fn unknown_or_wrong_side_color_theme_id_falls_back_to_side_default() {
    for (id, is_dark) in [
        ("unknown-light", false),
        ("unknown-dark", true),
        ("nord", false),
        ("solarized-light", true),
    ] {
        assert_eq!(
            ColorTheme::resolve(id, is_dark).id,
            ColorTheme::default_id(is_dark),
            "{id}"
        );
    }
}

#[test]
fn catalog_color_themes_have_unique_ids_names_and_meet_contrast_thresholds() {
    let themes = color_themes();
    let mut ids = std::collections::HashSet::new();
    let mut failures = Vec::new();
    for theme in &themes {
        assert!(ids.insert(theme.id.as_str()), "duplicate id: {}", theme.id);
        assert_ne!(theme.name, theme.id, "no catalog name for {}", theme.id);
        let c = &theme.colors;
        let mut checks = vec![
            ("context", c.text_context, c.context_bg, 4.5),
            ("added", c.text_added, c.added_bg, 4.5),
            ("removed", c.text_removed, c.removed_bg, 4.5),
            ("gutter", c.gutter_fg, c.gutter_bg, 3.0),
            ("dim", c.text_dim, c.context_bg, 4.5),
            ("find", c.find_match_fg, c.find_match_bg, 4.5),
            (
                "conflict header",
                c.conflict_header_fg,
                c.conflict_header_bg,
                4.5,
            ),
            (
                "conflict section",
                c.conflict_section_fg,
                c.conflict_section_bg,
                4.5,
            ),
        ];
        let line_bgs = [c.context_bg, c.added_bg, c.removed_bg].map(|bg| (bg, 4.5, 3.5));
        let word_bgs = [c.added_word_bg, c.removed_word_bg].map(|bg| (bg, 2.8, 2.3));
        for (bg, token_min, comment_min) in line_bgs.into_iter().chain(word_bgs) {
            checks.extend([
                ("keyword", c.tok_keyword, bg, token_min),
                ("string", c.tok_string, bg, token_min),
                ("number", c.tok_number, bg, token_min),
                ("type", c.tok_type, bg, token_min),
                ("comment", c.tok_comment, bg, comment_min),
            ]);
        }
        checks.extend([
            ("added word", c.text_added, c.added_word_bg, 2.8),
            ("removed word", c.text_removed, c.removed_word_bg, 2.8),
        ]);
        if theme.id != ColorTheme::default_id(theme.is_dark) {
            let r = &theme.repo;
            let seed = theme.seed;
            assert_eq!(r.navigation_bg, Some(seed.surface));
            let text = [
                ("workspace", r.workspace),
                ("bookmark", r.bookmark),
                ("change id prefix", r.change_id_prefix),
                ("commit id prefix", r.commit_id_prefix),
                ("file added", r.file_added),
                ("file removed", r.file_removed),
                ("file modified", r.file_modified),
                ("file renamed", r.file_renamed),
                ("file lfs", r.file_lfs),
                ("note accent", r.note_accent),
            ];
            let text_floors = [
                (seed.background, 4.5),
                (seed.surface, 4.5),
                (seed.selection, 3.0),
            ];
            for (bg, min_ratio) in text_floors {
                checks.extend(text.iter().map(|(label, fg)| (*label, *fg, bg, min_ratio)));
            }
            for bg in [seed.background, seed.surface] {
                checks.push(("graph line", r.graph_line, bg, 3.0));
            }
        }
        for (label, fg, bg, min_ratio) in checks {
            let ratio = contrast_ratio(fg, bg);
            if ratio < min_ratio {
                failures.push(format!(
                    "{} ({label}): {fg:06x} on {bg:06x} is {ratio:.2} < {min_ratio}",
                    theme.id
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
