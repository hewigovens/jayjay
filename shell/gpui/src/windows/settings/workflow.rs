use crate::app::config::AppConfig;
use gpui::{AnyElement, IntoElement, ParentElement, Styled, div, px};

use super::shared::{section_title, subsection_title, toggle_field};
use crate::app::theme::Theme;

pub(super) fn workflow_section(cfg: &AppConfig, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(16.))
        .child(section_title("Workflow", t))
        .child(subsection_title("Confirmations", t))
        .child(toggle_field(
            "Confirm before abandoning changes",
            !cfg.features.skip_abandon_confirmation,
            "Ask before abandoning a change.",
            |c| c.features.skip_abandon_confirmation ^= true,
            "workflow-confirm-abandon",
            t,
        ))
        .child(toggle_field(
            "Confirm before deleting a workspace",
            !cfg.features.skip_workspace_delete_confirmation,
            "Ask before forgetting a workspace and deleting its directory.",
            |c| c.features.skip_workspace_delete_confirmation ^= true,
            "workflow-confirm-workspace-delete",
            t,
        ))
        .child(toggle_field(
            "Confirm drag-to-rebase",
            cfg.features.confirm_drag_rebase,
            "Ask before rebasing a change by drag and drop.",
            |c| c.features.confirm_drag_rebase ^= true,
            "workflow-confirm-rebase",
            t,
        ))
        .child(subsection_title("Repository Behavior", t))
        .child(toggle_field(
            "Enable Git submodule support",
            cfg.diff.enable_git_submodule_support,
            "Track submodule pointer updates as commits.",
            |c| c.diff.enable_git_submodule_support ^= true,
            "workflow-sub",
            t,
        ))
        .into_any_element()
}
