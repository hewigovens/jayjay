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
            added_bg: 0x12261f,
            removed_bg: 0x2e1414,
            context_bg: 0x10131a,
            separator_bg: 0x292929,
            conflict_header_bg: 0x302412,
            conflict_section_bg: 0x211c14,
            conflict_content_bg: 0x1a1814,
            conflict_header_fg: 0xfbbf24,
            conflict_section_fg: 0xcc934d,
            conflict_stripe: 0xed9c2e,
            conflict_stripe_alpha: 0.78,
            added_word_bg: 0x207a38,
            removed_word_bg: 0x7f2424,
            gutter_bg: 0x0c0f14,
            gutter_fg: 0x737373,
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
            gutter_fg: 0x848b94,
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
}

pub fn diff_theme_colors(is_dark: bool) -> DiffThemeColors {
    if is_dark {
        DiffThemeColors::dark()
    } else {
        DiffThemeColors::light()
    }
}

/// Change/commit-id shortest-unique-prefix highlight (`0xRRGGBB`), a muted violet
/// — lighter on dark, deeper on light. Shared so both shells highlight identically.
pub fn change_id_prefix_color(is_dark: bool) -> u32 {
    if is_dark { 0x9b7fcf } else { 0x7c4fc2 }
}
