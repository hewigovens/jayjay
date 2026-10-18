use crate::app::config::AppConfig;
use gpui::{AnyElement, IntoElement, ParentElement, Styled, div, px};

use super::shared::{form_section, toggle_field};
use crate::app::theme::Theme;
use crate::ui::icons::glyph;

pub(super) fn workflow_section(cfg: &AppConfig, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(24.))
        .children([
            form_section(
                "Confirmations",
                [
                    toggle_field(
                        glyph::TRASH,
                        "Confirm before abandoning changes",
                        !cfg.features.skip_abandon_confirmation,
                        |c| c.features.skip_abandon_confirmation ^= true,
                        "workflow-confirm-abandon",
                        t,
                    ),
                    toggle_field(
                        glyph::FOLDER_MINUS,
                        "Confirm before deleting a workspace",
                        !cfg.features.skip_workspace_delete_confirmation,
                        |c| c.features.skip_workspace_delete_confirmation ^= true,
                        "workflow-confirm-workspace-delete",
                        t,
                    ),
                    toggle_field(
                        glyph::APP_WINDOW,
                        "Confirm drag-to-rebase",
                        cfg.features.confirm_drag_rebase,
                        |c| c.features.confirm_drag_rebase ^= true,
                        "workflow-confirm-rebase",
                        t,
                    ),
                ],
                t,
            ),
            form_section(
                "Repository Behavior",
                [toggle_field(
                    glyph::SUBMODULE,
                    "Enable Git submodule support",
                    cfg.diff.enable_git_submodule_support,
                    |c| c.diff.enable_git_submodule_support ^= true,
                    "workflow-sub",
                    t,
                )],
                t,
            ),
        ])
        .into_any_element()
}
