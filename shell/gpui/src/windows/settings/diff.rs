use crate::app::config::AppConfig;
use gpui::{AnyElement, IntoElement, ParentElement, Styled, div, px};

use super::shared::{section_title, toggle_field};
use crate::app::theme::Theme;

pub(super) fn diff_section(cfg: &AppConfig, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(section_title("Diff", t))
        .child(toggle_field(
            "Side-by-side by default",
            cfg.diff.side_by_side,
            "Open file diffs in a two-column layout.",
            |c| c.diff.side_by_side ^= true,
            "diff-sbs",
            t,
        ))
        .child(toggle_field(
            "Ignore whitespace",
            cfg.diff.ignore_whitespace,
            "Skip whitespace-only changes in diff output.",
            |c| c.diff.ignore_whitespace ^= true,
            "diff-ws",
            t,
        ))
        .child(toggle_field(
            "Hide Git LFS placeholders",
            cfg.diff.hide_git_lfs,
            "Replace LFS pointers with a placeholder card.",
            |c| c.diff.hide_git_lfs ^= true,
            "diff-lfs",
            t,
        ))
        .child(toggle_field(
            "Show submodule changes",
            cfg.diff.enable_git_submodule_support,
            "Track submodule pointer updates as commits.",
            |c| c.diff.enable_git_submodule_support ^= true,
            "diff-sub",
            t,
        ))
        .child(toggle_field(
            "Tree-style file list",
            cfg.diff.tree_file_list,
            "Group files by directory in the file column.",
            |c| c.diff.tree_file_list ^= true,
            "diff-tree",
            t,
        ))
        .into_any_element()
}
