use jayjay_core::{CliStatus, check_gh_environment, check_glab_environment, check_jj_environment};

use crate::app::config::AppConfig;
use crate::app::tools::{EDITOR_OPTIONS, TERMINAL_OPTIONS};
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, div,
    px,
};

use super::SettingsView;
use super::dropdown::dropdown_button;
use super::shared::{
    current_value, detail_row, feedback_copy_icon_button, field_row, section_title,
    subsection_title,
};
use crate::app::theme::Theme;
use crate::platform::{CUSTOM_TERMINAL_HINT, CUSTOM_TERMINAL_LABEL};
use crate::ui::icons::{self, glyph};

const JJ_TOOL_CONFIG_COPY_ID: &str = "settings-copy-jj-tool-config";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AiToolStatuses {
    pub codex: Option<String>,
    pub claude: Option<String>,
    pub jayjay: Option<String>,
}

pub(super) fn load_ai_tool_statuses() -> AiToolStatuses {
    AiToolStatuses {
        codex: jayjay_core::find_existing_binary("codex"),
        claude: jayjay_core::find_existing_binary("claude"),
        jayjay: jayjay_core::find_existing_binary("jayjay"),
    }
}

impl SettingsView {
    /// Detection seam for component tests: replaces the async-loaded snapshot and wins over any load still in flight.
    pub fn set_ai_tool_statuses(&mut self, statuses: AiToolStatuses, cx: &mut Context<Self>) {
        self.ai_tools = Some(statuses);
        self.tools_loading = false;
        cx.notify();
    }
}

pub(super) fn tools_section(
    cfg: &AppConfig,
    ai_tools: Option<&AiToolStatuses>,
    cli_install: Option<Option<&crate::app::cli_install::CliInstallState>>,
    recently_copied: Option<&SharedString>,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let mut section = div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(16.))
        .child(section_title("Tools", t))
        .child(field_row(
            "External editor",
            dropdown_button(
                "editor",
                dropdown_label(EDITOR_OPTIONS, &cfg.tools.external_editor),
                t,
                cx,
            ),
            "Used by 'Open in Editor' actions.",
            t,
        ));
    if cfg.tools.external_editor == "custom" {
        section = section.child(field_row(
            "Command",
            current_value(setting_value(&cfg.tools.custom_editor_command), t),
            "e.g. code, nvim",
            t,
        ));
    }
    section = section.child(field_row(
        "Terminal",
        dropdown_button(
            "terminal",
            dropdown_label(TERMINAL_OPTIONS, &cfg.tools.terminal),
            t,
            cx,
        ),
        "Used by 'Open in Terminal'.",
        t,
    ));
    if cfg.tools.terminal == "custom" {
        section = section.child(field_row(
            CUSTOM_TERMINAL_LABEL,
            current_value(setting_value(&cfg.tools.custom_terminal_command), t),
            CUSTOM_TERMINAL_HINT,
            t,
        ));
    }
    section =
        section
            .child(ai_tool_rows(ai_tools, t))
            .child(cli_tools(ai_tools, recently_copied, t, cx));
    if let Some(rows) = super::cli_row::command_line_rows(cli_install, t, cx) {
        section = section.child(rows);
    }
    section.into_any_element()
}

fn setting_value(value: &str) -> &str {
    if value.is_empty() { "(none)" } else { value }
}

fn dropdown_label(options: &[(&str, &str)], current: &str) -> String {
    options
        .iter()
        .find(|(id, _)| *id == current)
        .map(|(_, label)| (*label).to_owned())
        .unwrap_or_else(|| current.to_owned())
}

fn ai_tool_rows(ai_tools: Option<&AiToolStatuses>, t: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(2.))
        .child(subsection_title("AI Commit Message", t))
        .child(binary_row(
            "Codex CLI",
            glyph::FILE_CODE,
            ai_tools.map(|s| s.codex.as_deref()),
            "Not found",
            t,
        ))
        .child(binary_row(
            "Claude CLI",
            glyph::SPARKLE,
            ai_tools.map(|s| s.claude.as_deref()),
            "Not found",
            t,
        ))
}

fn cli_tools(
    ai_tools: Option<&AiToolStatuses>,
    recently_copied: Option<&SharedString>,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(2.))
        .child(subsection_title("CLI", t))
        .child(binary_row(
            "jayjay",
            glyph::INFO,
            ai_tools.map(|s| s.jayjay.as_deref()),
            "Not installed",
            t,
        ))
        .child(
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
        .child(cli_row("jj", glyph::GIT_BRANCH, check_jj_environment(), t))
        .child(cli_row("gh", glyph::GIT_MERGE, check_gh_environment(), t))
        .child(cli_row(
            "glab",
            glyph::GIT_MERGE,
            check_glab_environment(),
            t,
        ))
}

/// `None` while detection runs; found rows show the resolved binary path like the CLI rows below.
fn binary_row(
    name: &'static str,
    glyph_str: &'static str,
    resolved: Option<Option<&str>>,
    missing_label: &'static str,
    t: &Theme,
) -> impl IntoElement {
    let (detail, state): (SharedString, ToolState) = match resolved {
        None => ("Checking…".into(), ToolState::Checking),
        Some(Some(path)) => (path.to_owned().into(), ToolState::Found),
        Some(None) => (missing_label.into(), ToolState::Missing),
    };
    status_row(name, glyph_str, detail, state, t)
}

fn cli_row(
    name: &'static str,
    glyph_str: &'static str,
    status: CliStatus,
    t: &Theme,
) -> impl IntoElement {
    let detail = if status.is_installed {
        if status.version.is_empty() {
            status.path
        } else {
            format!("{name} {}", status.version)
        }
    } else {
        "Not installed".to_owned()
    };
    let state = if status.is_installed {
        ToolState::Found
    } else {
        ToolState::Missing
    };
    status_row(name, glyph_str, detail, state, t)
}

enum ToolState {
    Checking,
    Found,
    Missing,
}

fn status_row(
    name: &'static str,
    glyph_str: &'static str,
    detail: impl Into<SharedString>,
    state: ToolState,
    t: &Theme,
) -> impl IntoElement {
    let (icon_glyph, icon_color, detail_color, marker) = match state {
        ToolState::Checking => (glyph::ARROW_CLOCKWISE, t.fg_faint, t.fg_faint, "checking"),
        ToolState::Found => (glyph::CHECK, t.tag_added_fg, t.fg_dim, "found"),
        ToolState::Missing => (glyph::X_CIRCLE, t.fg_faint, t.fg_faint, "missing"),
    };
    detail_row(glyph_str, name, detail, 11., detail_color, t)
        .debug_selector(move || format!("settings-tool-row-{name}"))
        .child(
            // State-suffixed marker so component tests can assert found/missing/checking without reading text.
            icons::icon(icon_glyph, 13., icon_color)
                .debug_selector(move || format!("settings-tool-state-{name}-{marker}")),
        )
}
