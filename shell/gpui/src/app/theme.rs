use gpui::{App, Context, Global, Window, WindowAppearance};

mod palette;

#[derive(Clone, Debug)]
pub struct Theme {
    pub(crate) is_dark: bool,

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
    #[allow(dead_code)]
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

pub(crate) fn theme(cx: &App) -> &Theme {
    cx.global::<Theme>()
}

pub(crate) fn with_alpha(color: u32, alpha: u8) -> u32 {
    ((color & 0x00ff_ffff) << 8) | u32::from(alpha)
}

fn refresh_for_appearance(cx: &mut App, system: WindowAppearance) {
    let mode = crate::app::config::current(cx).appearance;
    cx.set_global(Theme::for_appearance(mode, system));
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
