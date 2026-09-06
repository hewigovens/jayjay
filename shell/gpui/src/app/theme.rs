use gpui::{App, Context, Global, Rems, Window, WindowAppearance, px, rems};
use jayjay_core::diff::syntax::SyntaxToken;

mod palette;

#[derive(Clone, Debug)]
pub struct Theme {
    pub(crate) is_dark: bool,

    pub(crate) font_size: f32,

    pub(crate) sidebar_bg: u32,
    pub(crate) detail_bg: u32,
    pub(crate) header_bg: u32,
    pub(crate) row_alt_bg: u32,
    pub(crate) selected_bg: u32,

    pub(crate) fg: u32,
    pub(crate) fg_dim: u32,
    pub(crate) fg_faint: u32,

    pub(crate) border: u32,
    pub(crate) row_border: u32,

    pub(crate) selected_accent: u32,
    pub(crate) success_fg: u32,
    pub wc_accent: u32,
    pub(crate) compare_bg: u32,
    pub(crate) compare_accent: u32,
    pub(crate) dag_line: u32,
    pub(crate) dag_edge: u32,
    pub(crate) dag_node: u32,

    pub(crate) tag_bg: u32,
    pub(crate) tag_fg: u32,
    pub(crate) tag_wc_bg: u32,
    pub(crate) tag_wc_fg: u32,
    pub(crate) tag_conflict_bg: u32,
    pub(crate) tag_conflict_fg: u32,
    pub(crate) tag_divergent_bg: u32,
    pub(crate) tag_divergent_fg: u32,
    pub(crate) tag_bookmark_bg: u32,
    pub(crate) tag_bookmark_fg: u32,
    pub(crate) tag_bookmark_icon: u32,
    pub(crate) change_id_prefix: u32,
    pub(crate) tag_tag_bg: u32,
    pub(crate) tag_tag_fg: u32,
    pub(crate) tag_tag_icon: u32,

    pub(crate) diff_added_bg: u32,
    pub(crate) diff_removed_bg: u32,
    pub(crate) diff_context_bg: u32,
    pub(crate) diff_separator_bg: u32,
    pub(crate) diff_conflict_header_bg: u32,
    pub(crate) diff_conflict_section_bg: u32,
    pub(crate) diff_conflict_content_bg: u32,
    pub(crate) diff_conflict_header_fg: u32,
    pub(crate) diff_conflict_section_fg: u32,
    pub(crate) diff_conflict_stripe: u32,
    pub(crate) diff_added_word_bg: u32,
    pub(crate) diff_removed_word_bg: u32,
    pub(crate) diff_gutter_bg: u32,
    pub(crate) diff_gutter_fg: u32,
    pub(crate) diff_gutter_added_fg: u32,
    pub(crate) diff_gutter_removed_fg: u32,
    pub(crate) diff_text_context: u32,
    pub(crate) diff_text_added: u32,
    pub(crate) diff_text_removed: u32,
    pub(crate) diff_text_dim: u32,

    pub(crate) tok_keyword: u32,
    pub(crate) tok_string: u32,
    pub(crate) tok_comment: u32,
    pub(crate) tok_number: u32,
    pub(crate) tok_type: u32,

    pub(crate) tag_added_bg: u32,
    pub(crate) tag_added_fg: u32,
    pub(crate) tag_removed_bg: u32,
    pub(crate) tag_removed_fg: u32,
    pub(crate) tag_modified_bg: u32,
    pub(crate) tag_modified_fg: u32,
    pub(crate) tag_renamed_bg: u32,
    pub(crate) tag_renamed_fg: u32,

    pub(crate) file_added_color: u32,
    pub(crate) file_removed_color: u32,
    pub(crate) file_modified_color: u32,
    pub(crate) file_renamed_color: u32,
    pub(crate) file_lfs_color: u32,

    pub(crate) error_fg: u32,

    pub(crate) find_match_bg: u32,
    pub(crate) find_match_fg: u32,

    pub(crate) toggle_active_bg: u32,
    pub(crate) toggle_active_fg: u32,
    pub(crate) toggle_inactive_bg: u32,
    pub(crate) toggle_inactive_fg: u32,

    pub(crate) toolbar_bg: u32,
    pub(crate) toolbar_group_bg: u32,
}

impl Global for Theme {}

// Match GPUI's native rem so the default setting leaves existing rem-based components unchanged.
const DEFAULT_REM_SIZE: f32 = 16.;

impl Theme {
    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    pub(crate) fn scaled_font_size(&self, base: f32) -> f32 {
        base * (self.font_size / crate::app::config::AppConfig::DEFAULT_FONT_SIZE)
    }

    pub(crate) fn scaled_control_height(&self, height: f32, base_font_size: f32) -> f32 {
        height + (self.scaled_font_size(base_font_size) - base_font_size).max(0.)
    }

    pub(crate) fn code_line_height(&self) -> f32 {
        (self.font_size + 5.).max(18.)
    }

    pub(crate) fn compact_code_font_size(&self) -> f32 {
        (self.font_size - 1.).max(9.)
    }

    pub(crate) fn syntax_token_color(&self, token: SyntaxToken) -> Option<u32> {
        match token {
            SyntaxToken::Keyword | SyntaxToken::Operator => Some(self.tok_keyword),
            SyntaxToken::StringLit => Some(self.tok_string),
            SyntaxToken::Comment => Some(self.tok_comment),
            SyntaxToken::Number => Some(self.tok_number),
            SyntaxToken::Type | SyntaxToken::Function | SyntaxToken::Attribute => {
                Some(self.tok_type)
            }
            SyntaxToken::Plain | SyntaxToken::Variable | SyntaxToken::Punctuation => None,
        }
    }
}

pub(crate) const fn ui_font_size(base: f32) -> Rems {
    rems(base / DEFAULT_REM_SIZE)
}

pub(crate) fn theme_for_window<'a>(window: &mut Window, cx: &'a App) -> &'a Theme {
    let theme = theme(cx);
    window.set_rem_size(px(theme.scaled_font_size(DEFAULT_REM_SIZE)));
    theme
}

pub(crate) fn theme(cx: &App) -> &Theme {
    cx.global::<Theme>()
}

pub(crate) fn with_alpha(color: u32, alpha: u8) -> u32 {
    ((color & 0x00ff_ffff) << 8) | u32::from(alpha)
}

fn refresh_for_appearance(cx: &mut App, system: WindowAppearance) {
    let config = crate::app::config::current(cx);
    cx.set_global(
        Theme::for_appearance(config.appearance, system).with_font_size(config.font_size),
    );
    cx.refresh_windows();
}

pub(crate) fn refresh_for_current_appearance(cx: &mut App) {
    refresh_for_appearance(cx, cx.window_appearance());
}

pub fn observe_window_appearance<T: 'static>(window: &mut Window, cx: &mut Context<T>) {
    refresh_for_appearance(cx, window.appearance());
    cx.observe_window_appearance(window, |_, window, cx| {
        refresh_for_appearance(cx, window.appearance());
    })
    .detach();
}

pub(crate) const FONT_TAG: f32 = 10.;
pub(crate) const FONT_META: f32 = 10.;
pub(crate) const FONT_ID: f32 = 11.;
pub(crate) const FONT_BODY: f32 = 13.;

/// Order is load-bearing: hashing change-id bytes indexes into this array, so reordering it reassigns colors already shown for existing changes.
pub(crate) const ANNOTATE_PALETTE: &[u32] = &[
    0x4a5568, 0x6b46c1, 0x2563eb, 0x059669, 0xd97706, 0xdc2626, 0xdb2777, 0x0891b2, 0x7c3aed,
    0x84cc16, 0x06b6d4, 0xeab308,
];

#[cfg(test)]
mod tests;
