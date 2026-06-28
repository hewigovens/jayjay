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
        .child(section_title("Diff", t))
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
        .child(subsection_title("Git", t))
        .child(toggle_field(
            "Hide Git LFS-backed files",
            cfg.diff.hide_git_lfs,
            "Replace LFS pointers with a placeholder card.",
            |c| c.diff.hide_git_lfs ^= true,
            "diff-lfs",
            t,
        ))
        .child(toggle_field(
            "Enable Git submodule support",
            cfg.diff.enable_git_submodule_support,
            "Track submodule pointer updates as commits.",
            |c| c.diff.enable_git_submodule_support ^= true,
            "diff-sub",
            t,
        ))
        .child(subsection_title("Confirmations", t))
        .child(toggle_field(
            "Skip abandon confirmation",
            cfg.features.skip_abandon_confirmation,
            "Don't prompt before abandoning a change.",
            |c| c.features.skip_abandon_confirmation ^= true,
            "diff-confirm-abandon",
            t,
        ))
        .child(toggle_field(
            "Confirm drag-to-rebase",
            cfg.features.confirm_drag_rebase,
            "Ask before rebasing a change by drag and drop.",
            |c| c.features.confirm_drag_rebase ^= true,
            "diff-confirm-rebase",
            t,
        ))
        .into_any_element()
}
