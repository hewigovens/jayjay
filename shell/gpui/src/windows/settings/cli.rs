use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, div,
    px,
};

use super::SettingsView;
use super::cli_diagnostics::CliDiagnostics;
use super::cli_row;
use super::shared::{detail_row, feedback_copy_icon_button, form_section};
use super::tools::{AiToolStatuses, binary_row, detected_cli_row, forge_login_row};
use crate::app::cli_install;
use crate::app::theme::Theme;
use crate::ui::icons::glyph;

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
        .child(forge_rows(diagnostics, t))
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
                "Not installed",
                t,
            )
            .into_any_element(),
        );
    }
    rows.push(
        detail_row(
            glyph::FILE_CODE,
            "jj tool configuration",
            "diff, edit & merge",
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
    form_section("Version Control", rows, t).into_any_element()
}

fn forge_rows(diagnostics: Option<&CliDiagnostics>, t: &Theme) -> impl IntoElement {
    let mut rows: Vec<AnyElement> = jayjay_core::RepoHost::TOKEN_HOSTS
        .into_iter()
        .map(|host| {
            let source = diagnostics.map(|d| {
                d.forge_logins
                    .iter()
                    .find(|(login, _)| *login == host)
                    .and_then(|(_, source)| *source)
            });
            forge_login_row(host, source, t).into_any_element()
        })
        .collect();
    rows.push(
        detected_cli_row(
            "origin",
            glyph::GIT_MERGE,
            diagnostics.map(|d| &d.origin),
            t,
        )
        .into_any_element(),
    );
    form_section("Forges", rows, t)
}
