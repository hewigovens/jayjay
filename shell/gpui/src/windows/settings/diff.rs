use crate::app::config::AppConfig;
use gpui::{AnyElement, IntoElement, ParentElement, Styled, div, px};

use super::shared::{form_section, toggle_field};
use crate::app::theme::Theme;
use crate::ui::icons::glyph;

pub(super) fn diff_section(cfg: &AppConfig, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(24.))
        .children([
            form_section(
                "Display",
                [
                    toggle_field(
                        glyph::COLUMNS,
                        "Side-by-side diff",
                        cfg.diff.side_by_side,
                        |c| c.diff.side_by_side ^= true,
                        "diff-sbs",
                        t,
                    ),
                    toggle_field(
                        glyph::WHITESPACE,
                        "Ignore whitespace changes",
                        cfg.diff.ignore_whitespace,
                        |c| c.diff.ignore_whitespace ^= true,
                        "diff-ws",
                        t,
                    ),
                    toggle_field(
                        glyph::LIST_TREE,
                        "Tree view for files",
                        cfg.diff.tree_file_list,
                        |c| c.diff.tree_file_list ^= true,
                        "diff-tree",
                        t,
                    ),
                    toggle_field(
                        glyph::EYE_OFF,
                        "Hide reviewed files",
                        cfg.diff.hide_reviewed_files,
                        |c| c.diff.hide_reviewed_files ^= true,
                        "diff-hide-reviewed",
                        t,
                    ),
                    toggle_field(
                        glyph::UNFOLD_VERTICAL,
                        "Auto-expand descriptions",
                        cfg.diff.auto_expand_description,
                        |c| c.diff.auto_expand_description ^= true,
                        "diff-auto-expand",
                        t,
                    ),
                ],
                t,
            ),
            form_section(
                "Large Files",
                [toggle_field(
                    glyph::DATABASE,
                    "Hide Git LFS-backed files",
                    cfg.diff.hide_git_lfs,
                    |c| c.diff.hide_git_lfs ^= true,
                    "diff-lfs",
                    t,
                )],
                t,
            ),
        ])
        .into_any_element()
}
