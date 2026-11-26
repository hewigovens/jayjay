use gpui::{AnyElement, Context, IntoElement, ParentElement, Styled, div};

use super::{SettingsView, cli, forges, tools};
use crate::app::config::AppConfig;
use crate::app::theme::{Theme, ui_size};

pub(super) fn integrations_section(
    view: &SettingsView,
    cfg: &AppConfig,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(ui_size(24.))
        .child(tools::tool_sections(
            cfg,
            view.ai_tools.as_ref(),
            &view.custom_editor_command,
            &view.custom_terminal_command,
            t,
            cx,
        ))
        .child(cli::cli_sections(
            view.ai_tools.as_ref(),
            view.cli_diagnostics.as_ref(),
            view.cli_install.as_ref().map(Option::as_ref),
            view.recently_copied.as_ref(),
            t,
            cx,
        ))
        .child(forges::forge_section(view, t, cx))
        .into_any_element()
}
