use super::ThemeSeed;
use super::color::{mix, readable_on};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiffThemeColors {
    pub added_bg: u32,
    pub removed_bg: u32,
    pub context_bg: u32,
    pub separator_bg: u32,
    pub conflict_header_bg: u32,
    pub conflict_section_bg: u32,
    pub conflict_content_bg: u32,
    pub conflict_header_fg: u32,
    pub conflict_section_fg: u32,
    pub conflict_stripe: u32,
    pub conflict_stripe_alpha: f64,
    pub added_word_bg: u32,
    pub removed_word_bg: u32,
    pub gutter_bg: u32,
    pub gutter_fg: u32,
    pub gutter_added_fg: u32,
    pub gutter_removed_fg: u32,
    pub text_context: u32,
    pub text_added: u32,
    pub text_removed: u32,
    pub text_dim: u32,
    pub tok_keyword: u32,
    pub tok_string: u32,
    pub tok_comment: u32,
    pub tok_number: u32,
    pub tok_type: u32,
    pub group_stripe: u32,
    pub group_stripe_alpha: f64,
    pub find_match_bg: u32,
    pub find_match_fg: u32,
}

impl DiffThemeColors {
    pub fn dark() -> Self {
        Self {
            added_bg: 0x183c32,
            removed_bg: 0x48262e,
            context_bg: 0x10131a,
            separator_bg: 0x1a2230,
            conflict_header_bg: 0x302412,
            conflict_section_bg: 0x211c14,
            conflict_content_bg: 0x1a1814,
            conflict_header_fg: 0xfbbf24,
            conflict_section_fg: 0xcc934d,
            conflict_stripe: 0xed9c2e,
            conflict_stripe_alpha: 0.78,
            added_word_bg: 0x246347,
            removed_word_bg: 0x592b37,
            gutter_bg: 0x0c0f14,
            gutter_fg: 0x97a0af,
            gutter_added_fg: 0x32d74b,
            gutter_removed_fg: 0xff453a,
            text_context: 0xd9d9d9,
            text_added: 0x77e887,
            text_removed: 0xff7a73,
            text_dim: 0x8a8f99,
            tok_keyword: 0xff7a73,
            tok_string: 0xa6d6ff,
            tok_comment: 0x8c94a1,
            tok_number: 0x78bfff,
            tok_type: 0xd1a8ff,
            group_stripe: 0x6b9ee6,
            group_stripe_alpha: 0.55,
            find_match_bg: 0x854d0e,
            find_match_fg: 0xfde68a,
        }
    }

    pub fn light() -> Self {
        Self {
            added_bg: 0xdafbe1,
            removed_bg: 0xffebe9,
            context_bg: 0xffffff,
            separator_bg: 0xeef0f3,
            conflict_header_bg: 0xfff2d6,
            conflict_section_bg: 0xfffaeb,
            conflict_content_bg: 0xfffcf5,
            conflict_header_fg: 0x8a4a06,
            conflict_section_fg: 0x80571f,
            conflict_stripe: 0xed8a1e,
            conflict_stripe_alpha: 0.62,
            added_word_bg: 0xaceebb,
            removed_word_bg: 0xffcecb,
            gutter_bg: 0xf6f7f9,
            gutter_fg: 0x57606a,
            gutter_added_fg: 0x28cd41,
            gutter_removed_fg: 0xff3b30,
            text_context: 0x1f2328,
            text_added: 0x14532d,
            text_removed: 0x991b1b,
            text_dim: 0x57606a,
            tok_keyword: 0xcf222e,
            tok_string: 0x0a3069,
            tok_comment: 0x6e7781,
            tok_number: 0x0550ae,
            tok_type: 0x6f42c1,
            group_stripe: 0x5c94db,
            group_stripe_alpha: 0.42,
            find_match_bg: 0xfde68a,
            find_match_fg: 0x451a03,
        }
    }

    /// Custom palettes are derived; the built-in defaults retain their hand-tuned colors.
    pub fn from_seed(seed: &ThemeSeed) -> Self {
        let dark = seed.is_dark();
        let bg = seed.background;
        let fg = seed.foreground;
        let pick = |on_dark: f32, on_light: f32| if dark { on_dark } else { on_light };
        let added_bg = seed.tint(seed.green, pick(0.16, 0.18));
        let removed_bg = seed.tint(seed.red, pick(0.16, 0.11));
        let added_word_bg = seed.tint(seed.green, pick(0.45, 0.40));
        let removed_word_bg = seed.tint(seed.red, pick(0.45, 0.28));
        let token_floors = |line: f32, word: f32| {
            [
                (bg, line),
                (added_bg, line),
                (removed_bg, line),
                (added_word_bg, word),
                (removed_word_bg, word),
            ]
        };
        let token_color = |color| readable_on(color, fg, &token_floors(4.5, 2.8));
        let conflict_header_bg = seed.tint(seed.yellow, pick(0.16, 0.22));
        let conflict_section_bg = seed.tint(seed.yellow, pick(0.10, 0.12));
        let find_match_bg = seed.tint(seed.yellow, pick(0.45, 0.55));
        let gutter_bg = if dark {
            mix(bg, 0x000000, 0.2)
        } else {
            seed.surface
        };
        Self {
            added_bg,
            removed_bg,
            context_bg: bg,
            separator_bg: mix(bg, fg, pick(0.10, 0.06)),
            conflict_header_bg,
            conflict_section_bg,
            conflict_content_bg: seed.tint(seed.yellow, pick(0.05, 0.06)),
            conflict_header_fg: readable_on(
                mix(seed.yellow, fg, pick(0.15, 0.55)),
                fg,
                &[(conflict_header_bg, 4.5)],
            ),
            conflict_section_fg: readable_on(
                mix(seed.yellow, fg, pick(0.30, 0.60)),
                fg,
                &[(conflict_section_bg, 4.5)],
            ),
            conflict_stripe: seed.orange,
            conflict_stripe_alpha: if dark { 0.78 } else { 0.62 },
            added_word_bg,
            removed_word_bg,
            gutter_bg,
            gutter_fg: readable_on(seed.muted, fg, &[(gutter_bg, 3.0)]),
            gutter_added_fg: seed.green,
            gutter_removed_fg: seed.red,
            text_context: fg,
            text_added: readable_on(
                mix(seed.green, fg, pick(0.35, 0.55)),
                fg,
                &[(added_bg, 4.5), (added_word_bg, 2.8)],
            ),
            text_removed: readable_on(
                mix(seed.red, fg, pick(0.30, 0.50)),
                fg,
                &[(removed_bg, 4.5), (removed_word_bg, 2.8)],
            ),
            text_dim: readable_on(seed.muted, fg, &[(bg, 4.5)]),
            tok_keyword: token_color(mix(seed.red, fg, pick(0.30, 0.20))),
            tok_string: token_color(if dark {
                mix(seed.cyan, fg, 0.45)
            } else {
                mix(seed.blue, fg, 0.60)
            }),
            tok_comment: readable_on(seed.muted, fg, &token_floors(3.5, 2.3)),
            tok_number: token_color(mix(seed.blue, fg, pick(0.40, 0.35))),
            tok_type: token_color(mix(seed.magenta, fg, pick(0.30, 0.25))),
            group_stripe: seed.blue,
            group_stripe_alpha: if dark { 0.55 } else { 0.42 },
            find_match_bg,
            find_match_fg: readable_on(
                mix(seed.yellow, fg, pick(0.35, 0.80)),
                fg,
                &[(find_match_bg, 4.5)],
            ),
        }
    }
}
