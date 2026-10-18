use crate::app::config::AppConfig;
use crate::app::tools::{EDITOR_OPTIONS, TERMINAL_OPTIONS};
use gpui::{
    AnyElement, AppContext, Context, Div, Entity, InteractiveElement, IntoElement, ParentElement,
    SharedString, Styled, div, px,
};

use super::SettingsView;
use super::dropdown::dropdown_button;
use super::shared::{detail_row, field_row, form_section};
use crate::app::theme::Theme;
use crate::platform::CUSTOM_TERMINAL_LABEL;
use crate::ui::icons::{self, glyph};
use crate::ui::text_area::{TextArea, TextAreaUpdated};

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
        cx.notify();
    }
}

pub(super) fn tool_sections(
    cfg: &AppConfig,
    ai_tools: Option<&AiToolStatuses>,
    custom_editor_command: &Entity<TextArea>,
    custom_terminal_command: &Entity<TextArea>,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let mut rows = vec![field_row(
        glyph::BRACES,
        "Editor",
        dropdown_button(
            "editor",
            dropdown_label(EDITOR_OPTIONS, &cfg.tools.external_editor),
            t,
            cx,
        ),
        t,
    )];
    if cfg.tools.external_editor == "custom" {
        rows.push(field_row(
            glyph::TERMINAL,
            "Command",
            command_input(
                "setting-custom-editor-command",
                custom_editor_command.clone(),
            ),
            t,
        ));
    }
    rows.push(field_row(
        glyph::TERMINAL,
        "Terminal",
        dropdown_button(
            "terminal",
            dropdown_label(TERMINAL_OPTIONS, &cfg.tools.terminal),
            t,
            cx,
        ),
        t,
    ));
    if cfg.tools.terminal == "custom" {
        rows.push(field_row(
            glyph::TERMINAL,
            CUSTOM_TERMINAL_LABEL,
            command_input(
                "setting-custom-terminal-command",
                custom_terminal_command.clone(),
            ),
            t,
        ));
    }
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(24.))
        .child(form_section("Applications", rows, t))
        .child(ai_tool_rows(ai_tools, t))
        .into_any_element()
}

fn command_input(id: &'static str, input: Entity<TextArea>) -> AnyElement {
    div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .w(px(360.))
        .max_w_full()
        .min_w_0()
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
    form_section(
        "AI Commit Message",
        [
            binary_row(
                "Codex CLI",
                glyph::FILE_CODE,
                ai_tools.map(|s| s.codex.as_deref()),
                "Not found",
                t,
            )
            .into_any_element(),
            binary_row(
                "Claude CLI",
                glyph::SPARKLE,
                ai_tools.map(|s| s.claude.as_deref()),
                "Not found",
                t,
            )
            .into_any_element(),
        ],
        t,
    )
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
    status: Option<&jayjay_core::CliStatus>,
    t: &Theme,
) -> impl IntoElement {
    let (detail, state) = match status {
        Some(status) if status.is_installed => (
            if status.version.is_empty() {
                status.path.clone()
            } else {
                format!("{name} {}", status.version)
            },
            ToolState::Found,
        ),
        Some(_) => ("Not installed".to_owned(), ToolState::Missing),
        None => ("Checking…".to_owned(), ToolState::Checking),
    };
    status_row(name, glyph_str, detail, state, t)
}

pub(super) enum ToolState {
    Checking,
    Found,
    Missing,
}

pub(super) fn status_row(
    name: &'static str,
    glyph_str: &'static str,
    detail: impl Into<SharedString>,
    state: ToolState,
    t: &Theme,
) -> Div {
    let (icon_glyph, icon_color, detail_color, marker) = match state {
        ToolState::Checking => (glyph::SYNC, t.fg_faint, t.fg_faint, "checking"),
        ToolState::Found => (glyph::CHECK, t.tag_added_fg, t.fg_dim, "found"),
        ToolState::Missing => (glyph::X_CIRCLE, t.fg_faint, t.fg_faint, "missing"),
    };
    detail_row(glyph_str, name, detail, detail_color, t)
        .debug_selector(move || format!("settings-tool-row-{name}"))
        .child(
            icons::icon(icon_glyph, 13., icon_color)
                .debug_selector(move || format!("settings-tool-state-{name}-{marker}")),
        )
}

pub(super) fn persisted_command(
    initial: String,
    placeholder: &'static str,
    persist: fn(&mut crate::app::config::AppConfig, String),
    cx: &mut Context<SettingsView>,
) -> Entity<TextArea> {
    let input = cx.new(|cx| TextArea::new(initial, placeholder, false, 32., cx));
    cx.subscribe(&input, move |_, input, _: &TextAreaUpdated, cx| {
        let value = input.read(cx).text();
        crate::app::config::update(cx, |cfg| persist(cfg, value));
    })
    .detach();
    input
}
