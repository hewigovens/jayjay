use crate::app::config::AppearanceMode;
use gpui::{App, Context, Global, Window, WindowAppearance};

#[derive(Clone, Debug)]
pub struct Theme {
    pub is_dark: bool,

    // Surfaces
    pub sidebar_bg: u32,
    pub detail_bg: u32,
    pub header_bg: u32,
    pub status_bg: u32,
    pub row_alt_bg: u32,
    pub selected_bg: u32,

    // Text
    pub fg: u32,
    pub fg_dim: u32,
    pub fg_faint: u32,

    // Borders
    pub border: u32,
    pub row_border: u32,

    // Accents
    pub selected_accent: u32,
    pub success_fg: u32,
    #[allow(dead_code)]
    pub wc_accent: u32,
    pub compare_bg: u32,
    pub compare_accent: u32,
    pub dag_line: u32,
    pub dag_edge: u32,
    pub dag_node: u32,

    // Tag palette (capsule pill)
    pub tag_bg: u32,
    pub tag_fg: u32,
    pub tag_wc_bg: u32,
    pub tag_wc_fg: u32,
    pub tag_conflict_bg: u32,
    pub tag_conflict_fg: u32,
    pub tag_divergent_bg: u32,
    pub tag_divergent_fg: u32,
    pub tag_bookmark_bg: u32,
    pub tag_bookmark_fg: u32,

    // Diff
    pub diff_added_bg: u32,
    pub diff_removed_bg: u32,
    pub diff_context_bg: u32,
    pub diff_separator_bg: u32,
    pub diff_added_word_bg: u32,
    pub diff_removed_word_bg: u32,
    pub diff_gutter_bg: u32,
    pub diff_gutter_fg: u32,
    pub diff_gutter_added_fg: u32,
    pub diff_gutter_removed_fg: u32,
    pub diff_text_context: u32,
    pub diff_text_added: u32,
    pub diff_text_removed: u32,
    pub diff_text_dim: u32,

    // Diff syntax tokens
    pub tok_keyword: u32,
    pub tok_string: u32,
    pub tok_comment: u32,
    pub tok_number: u32,
    pub tok_type: u32,

    // File-type tag palette
    pub tag_added_bg: u32,
    pub tag_added_fg: u32,
    pub tag_removed_bg: u32,
    pub tag_removed_fg: u32,
    pub tag_modified_bg: u32,
    pub tag_modified_fg: u32,
    pub tag_renamed_bg: u32,
    pub tag_renamed_fg: u32,

    // Errors
    pub error_fg: u32,

    // Find / search
    pub find_match_bg: u32,
    pub find_match_fg: u32,

    // Toggle button
    pub toggle_active_bg: u32,
    pub toggle_active_fg: u32,
    pub toggle_inactive_bg: u32,
    pub toggle_inactive_fg: u32,

    // Toolbar
    pub toolbar_button_bg: u32,
    pub toolbar_icon_bg: u32,
}

impl Global for Theme {}

impl Theme {
    pub fn dark() -> Self {
        Self {
            is_dark: true,
            sidebar_bg: 0x10131a,
            detail_bg: 0x10131a,
            header_bg: 0x1a1f27,
            status_bg: 0x0c0f14,
            row_alt_bg: 0x171b22,
            selected_bg: 0x1f2a3d,
            fg: 0xe6e6e6,
            fg_dim: 0x8a8f99,
            fg_faint: 0x595e68,
            border: 0x252a33,
            row_border: 0x1f242c,
            selected_accent: 0x3b82f6,
            success_fg: 0x77e887,
            wc_accent: 0xf59e0b,
            compare_bg: 0x251a12,
            compare_accent: 0xfb923c,
            dag_line: 0x282d35,
            dag_edge: 0x343942,
            dag_node: 0x4d5159,
            tag_bg: 0x252a33,
            tag_fg: 0xc6cad1,
            tag_wc_bg: 0x2a3550,
            tag_wc_fg: 0x93c5fd,
            tag_conflict_bg: 0x4a1f1f,
            tag_conflict_fg: 0xfca5a5,
            tag_divergent_bg: 0x4a3010,
            tag_divergent_fg: 0xfde68a,
            tag_bookmark_bg: 0x2a2f38,
            tag_bookmark_fg: 0xb9bfca,
            diff_added_bg: 0x12261f,
            diff_removed_bg: 0x2e1414,
            diff_context_bg: 0x10131a,
            diff_separator_bg: 0x292929,
            diff_added_word_bg: 0x1a662e,
            diff_removed_word_bg: 0x8c1f1f,
            diff_gutter_bg: 0x0c0f14,
            diff_gutter_fg: 0x737373,
            diff_gutter_added_fg: 0x32d74b,
            diff_gutter_removed_fg: 0xff453a,
            diff_text_context: 0xd9d9d9,
            diff_text_added: 0x77e887,
            diff_text_removed: 0xff7a73,
            diff_text_dim: 0x8a8f99,
            tok_keyword: 0xff7a73,
            tok_string: 0xa6d6ff,
            tok_comment: 0x8c94a1,
            tok_number: 0x78bfff,
            tok_type: 0xd1a8ff,
            tag_added_bg: 0x14532d,
            tag_added_fg: 0x86efac,
            tag_removed_bg: 0x7f1d1d,
            tag_removed_fg: 0xfecaca,
            tag_modified_bg: 0x1e3a8a,
            tag_modified_fg: 0xbfdbfe,
            tag_renamed_bg: 0x78350f,
            tag_renamed_fg: 0xfde68a,
            error_fg: 0xff6b6b,
            find_match_bg: 0x854d0e,
            find_match_fg: 0xfde68a,
            toggle_active_bg: 0x2a3550,
            toggle_active_fg: 0xbfdbfe,
            toggle_inactive_bg: 0x252a33,
            toggle_inactive_fg: 0xc6cad1,
            toolbar_button_bg: 0x252a33,
            toolbar_icon_bg: 0x1d2129,
        }
    }

    pub fn light() -> Self {
        Self {
            is_dark: false,
            sidebar_bg: 0xffffff,
            detail_bg: 0xffffff,
            header_bg: 0xffffff,
            status_bg: 0xeceef2,
            row_alt_bg: 0xf0f2f5,
            selected_bg: 0xd9e6fa,
            fg: 0x1f2328,
            fg_dim: 0x57606a,
            fg_faint: 0x848b94,
            border: 0xd0d7de,
            row_border: 0xe2e6eb,
            selected_accent: 0x3b82f6,
            success_fg: 0x14532d,
            wc_accent: 0xea580c,
            compare_bg: 0xfff7ed,
            compare_accent: 0xf97316,
            dag_line: 0xd9d9d9,
            dag_edge: 0xcacaca,
            dag_node: 0xb8b8b8,
            tag_bg: 0xe6e9ee,
            tag_fg: 0x47525e,
            tag_wc_bg: 0xddeaff,
            tag_wc_fg: 0x1d4ed8,
            tag_conflict_bg: 0xfde0e0,
            tag_conflict_fg: 0x991b1b,
            tag_divergent_bg: 0xfde7c4,
            tag_divergent_fg: 0x854d0e,
            tag_bookmark_bg: 0xe6e9ee,
            tag_bookmark_fg: 0x4a5360,
            diff_added_bg: 0xddf5e2,
            diff_removed_bg: 0xfceeee,
            diff_context_bg: 0xffffff,
            diff_separator_bg: 0xeef0f3,
            diff_added_word_bg: 0x9bd9a8,
            diff_removed_word_bg: 0xf2a4a4,
            diff_gutter_bg: 0xf6f7f9,
            diff_gutter_fg: 0x848b94,
            diff_gutter_added_fg: 0x28cd41,
            diff_gutter_removed_fg: 0xff3b30,
            diff_text_context: 0x1f2328,
            diff_text_added: 0x14532d,
            diff_text_removed: 0x991b1b,
            diff_text_dim: 0x57606a,
            tok_keyword: 0xcf222e,
            tok_string: 0x0a3069,
            tok_comment: 0x6e7781,
            tok_number: 0x0550ae,
            tok_type: 0x6f42c1,
            tag_added_bg: 0xddf5e2,
            tag_added_fg: 0x14532d,
            tag_removed_bg: 0xfceeee,
            tag_removed_fg: 0x991b1b,
            tag_modified_bg: 0xddeaff,
            tag_modified_fg: 0x1d4ed8,
            tag_renamed_bg: 0xfde7c4,
            tag_renamed_fg: 0x854d0e,
            error_fg: 0xb00020,
            find_match_bg: 0xfde68a,
            find_match_fg: 0x451a03,
            toggle_active_bg: 0xddeaff,
            toggle_active_fg: 0x1d4ed8,
            toggle_inactive_bg: 0xe6e9ee,
            toggle_inactive_fg: 0x47525e,
            toolbar_button_bg: 0xe6e9ee,
            toolbar_icon_bg: 0xeef0f3,
        }
    }

    pub fn for_appearance(mode: AppearanceMode, system: WindowAppearance) -> Self {
        match mode {
            AppearanceMode::Light => Self::light(),
            AppearanceMode::Dark => Self::dark(),
            AppearanceMode::System => match system {
                WindowAppearance::Light | WindowAppearance::VibrantLight => Self::light(),
                WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::dark(),
            },
        }
    }
}

pub fn theme(cx: &App) -> &Theme {
    cx.global::<Theme>()
}

fn refresh_for_appearance(cx: &mut App, system: WindowAppearance) {
    let mode = crate::app::config::current(cx).appearance;
    cx.set_global(Theme::for_appearance(mode, system));
    cx.refresh_windows();
}

pub fn refresh_for_current_appearance(cx: &mut App) {
    refresh_for_appearance(cx, cx.window_appearance());
}

pub fn observe_window_appearance<T: 'static>(window: &mut Window, cx: &mut Context<T>) {
    refresh_for_appearance(cx, window.appearance());
    cx.observe_window_appearance(window, |_, window, cx| {
        refresh_for_appearance(cx, window.appearance());
    })
    .detach();
}

// Font sizes (theme-independent for now).
pub const FONT_TAG: f32 = 10.;
pub const FONT_META: f32 = 10.;
pub const FONT_ID: f32 = 11.;
pub const FONT_BODY: f32 = 13.;

/// Stable per-change-id palette used by the annotate view stripe column.
/// Hashing change_id bytes into this set gives a deterministic color for each
/// change, similar to GitHub blame.
pub const ANNOTATE_PALETTE: &[u32] = &[
    0x4a5568, 0x6b46c1, 0x2563eb, 0x059669, 0xd97706, 0xdc2626, 0xdb2777, 0x0891b2, 0x7c3aed,
    0x84cc16, 0x06b6d4, 0xeab308,
];

#[cfg(test)]
mod tests {
    use super::*;

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
}
