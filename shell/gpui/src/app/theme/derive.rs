use jayjay_core::theme::{DiffThemeColors, ThemeSeed, mix};

use super::Theme;
use crate::app::config::AppConfig;

impl Theme {
    pub(crate) fn from_seed(seed: &ThemeSeed, diff: DiffThemeColors) -> Self {
        let dark = seed.is_dark();
        let bg = seed.background;
        let fg = seed.foreground;
        let pick = |on_dark: f32, on_light: f32| if dark { on_dark } else { on_light };
        let tag_bg = mix(bg, fg, pick(0.09, 0.075));
        let tag_fg = mix(fg, bg, pick(0.15, 0.25));
        Self {
            is_dark: dark,
            font_size: AppConfig::DEFAULT_FONT_SIZE,

            sidebar_bg: bg,
            detail_bg: bg,
            header_bg: if dark { seed.surface } else { bg },
            row_alt_bg: mix(bg, fg, pick(0.06, 0.045)),
            selected_bg: seed.selection,

            fg,
            fg_dim: seed.muted,
            fg_faint: mix(fg, bg, 0.55),

            border: seed.border,
            row_border: mix(bg, fg, pick(0.08, 0.065)),

            selected_accent: seed.accent,
            success_fg: diff.text_added,
            wc_accent: if dark {
                seed.orange
            } else {
                mix(seed.orange, 0x000000, 0.1)
            },
            compare_bg: seed.tint(seed.orange, pick(0.14, 0.09)),
            compare_accent: seed.orange,
            dag_line: mix(bg, fg, pick(0.13, 0.15)),
            dag_edge: mix(bg, fg, pick(0.17, 0.21)),
            dag_node: mix(bg, fg, pick(0.25, 0.28)),

            tag_bg,
            tag_fg,
            tag_wc_bg: seed.tint(seed.blue, pick(0.25, 0.14)),
            tag_wc_fg: seed.ink(seed.blue),
            tag_conflict_bg: seed.tint(seed.red, pick(0.25, 0.14)),
            tag_conflict_fg: seed.ink(seed.red),
            tag_divergent_bg: seed.tint(seed.orange, pick(0.25, 0.16)),
            tag_divergent_fg: seed.orange,
            tag_bookmark_bg: tag_bg,
            tag_bookmark_fg: tag_fg,
            tag_bookmark_icon: seed.deep(seed.green),
            change_id_prefix: seed.ink(seed.magenta),
            tag_tag_bg: tag_bg,
            tag_tag_fg: tag_fg,
            tag_tag_icon: seed.deep(seed.blue),

            diff_added_bg: diff.added_bg,
            diff_removed_bg: diff.removed_bg,
            diff_context_bg: diff.context_bg,
            diff_separator_bg: diff.separator_bg,
            diff_conflict_header_bg: diff.conflict_header_bg,
            diff_conflict_section_bg: diff.conflict_section_bg,
            diff_conflict_content_bg: diff.conflict_content_bg,
            diff_conflict_header_fg: diff.conflict_header_fg,
            diff_conflict_section_fg: diff.conflict_section_fg,
            diff_conflict_stripe: diff.conflict_stripe,
            diff_added_word_bg: diff.added_word_bg,
            diff_removed_word_bg: diff.removed_word_bg,
            diff_gutter_bg: diff.gutter_bg,
            diff_gutter_fg: diff.gutter_fg,
            diff_gutter_added_fg: diff.gutter_added_fg,
            diff_gutter_removed_fg: diff.gutter_removed_fg,
            diff_text_context: diff.text_context,
            diff_text_added: diff.text_added,
            diff_text_removed: diff.text_removed,
            diff_text_dim: diff.text_dim,

            tok_keyword: diff.tok_keyword,
            tok_string: diff.tok_string,
            tok_comment: diff.tok_comment,
            tok_number: diff.tok_number,
            tok_type: diff.tok_type,

            tag_added_bg: seed.tint(seed.green, pick(0.35, 0.16)),
            tag_added_fg: seed.ink(seed.green),
            tag_removed_bg: seed.tint(seed.red, pick(0.40, 0.09)),
            tag_removed_fg: seed.ink(seed.red),
            tag_modified_bg: seed.tint(seed.orange, pick(0.35, 0.14)),
            tag_modified_fg: seed.ink(seed.orange),
            tag_renamed_bg: seed.tint(seed.blue, pick(0.40, 0.14)),
            tag_renamed_fg: seed.ink(seed.blue),

            file_added_color: seed.green,
            file_removed_color: seed.red,
            file_modified_color: seed.orange,
            file_renamed_color: seed.blue,
            file_lfs_color: seed.magenta,

            error_fg: if dark {
                mix(seed.red, fg, 0.25)
            } else {
                mix(seed.red, 0x000000, 0.3)
            },

            find_match_bg: diff.find_match_bg,
            find_match_fg: diff.find_match_fg,

            toggle_active_bg: seed.tint(seed.blue, pick(0.28, 0.14)),
            toggle_active_fg: seed.ink(seed.blue),
            toggle_inactive_bg: tag_bg,
            toggle_inactive_fg: tag_fg,

            toolbar_bg: if dark { seed.surface } else { bg },
            toolbar_group_bg: if dark {
                mix(seed.surface, fg, 0.05)
            } else {
                seed.surface
            },
        }
    }
}
