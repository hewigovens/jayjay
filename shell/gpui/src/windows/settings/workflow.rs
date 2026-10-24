use crate::app::config::AppConfig;
use gpui::{AnyElement, IntoElement, ParentElement, Styled, div, px};
use jayjay_i18n::settings;

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
                settings::confirmations(),
                [
                    toggle_field(
                        glyph::TRASH,
                        settings::confirm_abandon(),
                        !cfg.features.skip_abandon_confirmation,
                        |c| c.features.skip_abandon_confirmation ^= true,
                        "workflow-confirm-abandon",
                        t,
                    ),
                    toggle_field(
                        glyph::FOLDER_MINUS,
                        settings::confirm_workspace_delete(),
                        !cfg.features.skip_workspace_delete_confirmation,
                        |c| c.features.skip_workspace_delete_confirmation ^= true,
                        "workflow-confirm-workspace-delete",
                        t,
                    ),
                    toggle_field(
                        glyph::APP_WINDOW,
                        settings::confirm_drag_rebase(),
                        cfg.features.confirm_drag_rebase,
                        |c| c.features.confirm_drag_rebase ^= true,
                        "workflow-confirm-rebase",
                        t,
                    ),
                ],
                t,
            ),
            form_section(
                settings::repository_behavior(),
                [toggle_field(
                    glyph::SUBMODULE,
                    settings::submodule_support(),
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
