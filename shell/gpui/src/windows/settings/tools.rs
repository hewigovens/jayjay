use crate::app::config::AppConfig;
use crate::app::tools::{EDITOR_OPTIONS, TERMINAL_OPTIONS};
use gpui::{
    AnyElement, Context, Entity, InteractiveElement, IntoElement, ParentElement, SharedString,
    Styled, div, px,
};

use super::SettingsView;
use super::dropdown::dropdown_button;
use super::shared::{detail_row, field_row, section_title, subsection_title};
use crate::app::theme::Theme;
use crate::platform::{CUSTOM_TERMINAL_HINT, CUSTOM_TERMINAL_LABEL};
use crate::ui::icons::{self, glyph};
use crate::ui::text_area::TextArea;

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
    custom_editor_command: &Entity<TextArea>,
    custom_terminal_command: &Entity<TextArea>,
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
            command_input(
                "setting-custom-editor-command",
                custom_editor_command.clone(),
            ),
            "The executable and optional arguments used to open a path.",
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
            command_input(
                "setting-custom-terminal-command",
                custom_terminal_command.clone(),
            ),
            CUSTOM_TERMINAL_HINT,
            t,
        ));
    }
    section.child(ai_tool_rows(ai_tools, t)).into_any_element()
}

fn command_input(id: &'static str, input: Entity<TextArea>) -> AnyElement {
    div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .w(px(360.))
        .max_w(px(360.))
        .child(input)
        .into_any_element()
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

/// `None` while detection runs; found rows show the resolved binary path like the CLI rows below.
pub(super) fn binary_row(
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

pub(super) fn detected_cli_row(
    name: &'static str,
    glyph_str: &'static str,
    status: jayjay_core::CliStatus,
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
            icons::icon(icon_glyph, 13., icon_color)
                .debug_selector(move || format!("settings-tool-state-{name}-{marker}")),
        )
}
