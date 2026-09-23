use crate::app::config::AppConfig;
use gpui::{AnyElement, IntoElement, ParentElement, Styled, div, px};

use super::shared::{section_title, subsection_title, toggle_field};
use crate::app::theme::Theme;

pub(super) fn diff_section(cfg: &AppConfig, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(16.))
        .child(section_title("Diff & Files", t))
        .child(subsection_title("Display", t))
        .child(toggle_field(
            "Side-by-side diff",
            cfg.diff.side_by_side,
            "Open file diffs in a two-column layout.",
            |c| c.diff.side_by_side ^= true,
            "diff-sbs",
            t,
        ))
        .child(toggle_field(
            "Ignore whitespace changes",
            cfg.diff.ignore_whitespace,
            "Skip whitespace-only changes in diff output.",
            |c| c.diff.ignore_whitespace ^= true,
            "diff-ws",
            t,
        ))
        .child(toggle_field(
            "Tree view for files",
            cfg.diff.tree_file_list,
            "Group files by directory in the file column.",
            |c| c.diff.tree_file_list ^= true,
            "diff-tree",
            t,
        ))
        .child(toggle_field(
            "Hide reviewed files",
            cfg.diff.hide_reviewed_files,
            "Filter files you've already reviewed out of the file column.",
            |c| c.diff.hide_reviewed_files ^= true,
            "diff-hide-reviewed",
            t,
        ))
        .child(toggle_field(
            "Auto-expand descriptions",
            cfg.diff.auto_expand_description,
            "Show long change descriptions expanded instead of the compact preview.",
            |c| c.diff.auto_expand_description ^= true,
            "diff-auto-expand",
            t,
        ))
        .child(subsection_title("Large Files", t))
        .child(toggle_field(
            "Hide Git LFS-backed files",
            cfg.diff.hide_git_lfs,
            "Replace LFS pointers with a placeholder card.",
            |c| c.diff.hide_git_lfs ^= true,
            "diff-lfs",
            t,
        ))
        .into_any_element()
}
