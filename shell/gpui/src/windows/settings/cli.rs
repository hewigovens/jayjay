use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, div,
    px,
};

use super::SettingsView;
use super::cli_diagnostics::CliDiagnostics;
use super::cli_row;
use super::shared::{detail_row, feedback_copy_icon_button, subsection_title};
use super::tools::{AiToolStatuses, binary_row, detected_cli_row};
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
        .gap(px(16.))
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
    let mut col = div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(2.))
        .child(subsection_title("Version Control", t));
    if cli_install::supported() {
        if let Some(rows) = cli_row::command_line_rows(cli_install, t, cx) {
            col = col.child(rows);
        }
    } else {
        col = col.child(binary_row(
            "jayjay",
            glyph::INFO,
            ai_tools.map(|s| s.jayjay.as_deref()),
            "Not installed",
            t,
        ));
    }
    col.child(
        detail_row(
            glyph::FILE_CODE,
            "jj tool configuration",
            "diff, edit & merge",
            11.,
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
        )),
    )
    .child(detected_cli_row(
        "jj",
        glyph::GIT_BRANCH,
        diagnostics.map(|d| &d.jj),
        t,
    ))
    .into_any_element()
}

fn forge_rows(diagnostics: Option<&CliDiagnostics>, t: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(2.))
        .child(subsection_title("Forges", t))
        .child(detected_cli_row(
            "gh",
            glyph::GIT_MERGE,
            diagnostics.map(|d| &d.gh),
            t,
        ))
        .child(detected_cli_row(
            "glab",
            glyph::GIT_MERGE,
            diagnostics.map(|d| &d.glab),
            t,
        ))
        .child(detected_cli_row(
            "origin",
            glyph::GIT_MERGE,
            diagnostics.map(|d| &d.origin),
            t,
        ))
}
