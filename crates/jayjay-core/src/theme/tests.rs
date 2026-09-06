use super::*;

#[test]
fn mix_interpolates_per_channel_and_clamps() {
    assert_eq!(mix(0x000000, 0xffffff, 0.5), 0x808080);
    assert_eq!(mix(0xff0000, 0x0000ff, 0.), 0xff0000);
    assert_eq!(mix(0xff0000, 0x0000ff, 2.), 0x0000ff);
}

#[test]
fn luminance_separates_light_and_dark_backgrounds() {
    assert!(is_dark(0x10131a));
    assert!(is_dark(0x002b36));
    assert!(!is_dark(0xffffff));
    assert!(!is_dark(0xfdf6e3));
    assert!((luminance(0xffffff) - 1.).abs() < 1e-5);
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
fn diff_palette_derives_from_a_seed_with_readable_contrast() {
    for seed in [ThemeSeed::light(), ThemeSeed::dark()] {
        let diff = DiffThemeColors::from_seed(&seed);
        assert_eq!(diff.context_bg, seed.background);
        assert_ne!(diff.added_bg, diff.removed_bg);
        let bg_lum = luminance(seed.background);
        for text in [
            diff.text_context,
            diff.text_added,
            diff.text_removed,
            diff.tok_keyword,
        ] {
            assert!(
                (luminance(text) - bg_lum).abs() > 0.3,
                "{text:06x} on {:06x}",
                seed.background
            );
        }
    }
}

#[test]
fn builtin_diff_palettes_are_hand_tuned_not_derived() {
    assert_eq!(DiffThemeColors::light().added_bg, 0xdafbe1);
    assert_eq!(DiffThemeColors::dark().gutter_bg, 0x0c0f14);
}
