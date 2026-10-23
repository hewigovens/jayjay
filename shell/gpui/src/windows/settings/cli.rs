use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, div,
    px,
};

use super::SettingsView;
use super::cli_diagnostics::CliDiagnostics;
use super::cli_row;
use super::shared::{detail_row, feedback_copy_icon_button, form_section};
use super::tools::{AiToolStatuses, binary_row, detected_cli_row};
use crate::app::cli_install;
use crate::app::theme::Theme;
use crate::ui::icons::glyph;
use jayjay_i18n::{common, settings};

const JJ_TOOL_CONFIG_COPY_ID: &str = "settings-copy-jj-tool-config";

pub(super) fn cli_sections(
    ai_tools: Option<&AiToolStatuses>,
    diagnostics: Option<&CliDiagnostics>,
    cli_install: Option<Option<&crate::app::cli_install::CliInstallState>>,
    recently_copied: Option<&SharedString>,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    div()
        .debug_selector(|| "settings-cli-section".to_owned())
        .flex()
        .flex_col()
        .w_full()
        .gap(px(24.))
        .child(version_control_rows(
            ai_tools,
            diagnostics,
            cli_install,
            recently_copied,
            t,
            cx,
        ))
        .into_any_element()
}

fn version_control_rows(
    ai_tools: Option<&AiToolStatuses>,
    diagnostics: Option<&CliDiagnostics>,
    cli_install: Option<Option<&crate::app::cli_install::CliInstallState>>,
    recently_copied: Option<&SharedString>,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let mut rows = Vec::new();
    if cli_install::supported() {
        if let Some(row) = cli_row::command_line_rows(cli_install, t, cx) {
            rows.push(row);
        }
    } else {
        rows.push(
            binary_row(
                "jayjay",
                glyph::INFO,
                ai_tools.map(|s| s.jayjay.as_deref()),
                common::not_installed(),
                t,
            )
            .into_any_element(),
        );
    }
    rows.push(
        detail_row(
            glyph::FILE_CODE,
            settings::jj_tool_config(),
            settings::jj_tool_config_detail(),
            t.fg_dim,
            t,
        )
        .debug_selector(|| "settings-jj-tool-config-row".to_string())
        .child(feedback_copy_icon_button(
            JJ_TOOL_CONFIG_COPY_ID,
            jayjay_core::JJ_TOOL_CONFIG,
            recently_copied.is_some_and(|id| id.as_ref() == JJ_TOOL_CONFIG_COPY_ID),
            t,
            cx,
        ))
        .into_any_element(),
    );
    rows.push(
        detected_cli_row("jj", glyph::GIT_BRANCH, diagnostics.map(|d| &d.jj), t).into_any_element(),
    );
    form_section(settings::version_control(), rows, t).into_any_element()
}
