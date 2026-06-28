use std::process::Command;

use jayjay_core::{CliStatus, check_gh_environment, check_glab_environment, check_jj_environment};

use crate::app::config::AppConfig;
use crate::app::tools::{EDITOR_OPTIONS, TERMINAL_OPTIONS};
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, SharedString, Styled, div, px, rgb,
};

use super::SettingsView;
use super::shared::{current_value, detail_row, field_row, section_title, subsection_title};
use crate::app::theme::Theme;
use crate::platform::{CUSTOM_TERMINAL_HINT, CUSTOM_TERMINAL_LABEL};
use crate::ui::icons::{self, glyph};

/// Cached so the settings pane does not shell out on every render.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct AiToolStatuses {
    pub(super) codex: bool,
    pub(super) claude: bool,
    pub(super) jayjay: bool,
}

pub(super) fn load_ai_tool_statuses() -> AiToolStatuses {
    AiToolStatuses {
        codex: command_exists("codex"),
        claude: command_exists("claude"),
        jayjay: command_exists("jayjay"),
    }
}

pub(super) fn tools_section(
    cfg: &AppConfig,
    ai_tools: Option<&AiToolStatuses>,
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
            dropdown_button("editor", EDITOR_OPTIONS, &cfg.tools.external_editor, t, cx),
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
        dropdown_button("terminal", TERMINAL_OPTIONS, &cfg.tools.terminal, t, cx),
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
    section
        .child(ai_tool_rows(ai_tools, t))
        .child(cli_tools(ai_tools, t))
        .into_any_element()
}

fn setting_value(value: &str) -> &str {
    if value.is_empty() { "(none)" } else { value }
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
            ai_tools.map(|s| s.codex),
            "Installed",
            "Not found",
            t,
        ))
        .child(binary_row(
            "Claude CLI",
            glyph::SPARKLE,
            ai_tools.map(|s| s.claude),
            "Installed",
            "Not found",
            t,
        ))
}

fn cli_tools(ai_tools: Option<&AiToolStatuses>, t: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(2.))
        .child(subsection_title("CLI", t))
        .child(binary_row(
            "jayjay",
            glyph::INFO,
            ai_tools.map(|s| s.jayjay),
            "Installed",
            "Not installed",
            t,
        ))
        .child(cli_row("jj", glyph::GIT_BRANCH, check_jj_environment(), t))
        .child(cli_row("gh", glyph::GIT_MERGE, check_gh_environment(), t))
        .child(cli_row(
            "glab",
            glyph::GIT_MERGE,
            check_glab_environment(),
            t,
        ))
}

fn binary_row(
    name: &'static str,
    glyph_str: &'static str,
    installed: Option<bool>,
    installed_label: &'static str,
    missing_label: &'static str,
    t: &Theme,
) -> impl IntoElement {
    match installed {
        None => status_row(name, glyph_str, "Checking…", ToolState::Checking, t),
        Some(true) => status_row(name, glyph_str, installed_label, ToolState::Found, t),
        Some(false) => status_row(name, glyph_str, missing_label, ToolState::Missing, t),
    }
}

/// Resolves `command` the same way the rest of the app locates CLI binaries
/// (login-shell PATH, `~/.local/bin`, `~/.cargo/bin`, common install prefixes),
/// since packaged GUI apps don't inherit the user's shell PATH.
fn command_exists(command: &str) -> bool {
    let Some(resolved) = jayjay_core::find_existing_binary(command) else {
        return false;
    };
    Command::new(&resolved)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
        || Command::new(&resolved)
            .arg("version")
            .output()
            .is_ok_and(|output| output.status.success())
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
    let (icon_glyph, icon_color, detail_color) = match state {
        ToolState::Checking => (glyph::ARROW_CLOCKWISE, t.fg_faint, t.fg_faint),
        ToolState::Found => (glyph::CHECK, t.tag_added_fg, t.fg_dim),
        ToolState::Missing => (glyph::X_CIRCLE, t.fg_faint, t.fg_faint),
    };
    detail_row(glyph_str, name, detail, 11., detail_color, t)
        .debug_selector(move || format!("settings-tool-row-{name}"))
        .child(icons::icon(icon_glyph, 13., icon_color))
}

fn dropdown_button(
    field_id: &'static str,
    options: &'static [(&'static str, &'static str)],
    current: &str,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let label = options
        .iter()
        .find(|(id, _)| *id == current)
        .map(|(_, l)| *l)
        .unwrap_or(current);
    div()
        .id(SharedString::from(format!("dd-btn-{field_id}")))
        .debug_selector(move || format!("dd-btn-{field_id}"))
        .relative()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .h(px(32.))
        .pl(px(24.))
        .pr(px(40.))
        .rounded_lg()
        .bg(rgb(t.toggle_inactive_bg))
        .text_size(px(13.))
        .text_color(rgb(t.fg))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |view, ev: &MouseDownEvent, _w, cx| {
                view.open_dropdown(SharedString::from(field_id), ev.position, cx);
            }),
        )
        .child(
            div()
                .min_w_0()
                .text_center()
                .truncate()
                .child(SharedString::from(label.to_owned())),
        )
        .child(dropdown_chevrons(t))
        .into_any_element()
}

fn dropdown_chevrons(t: &Theme) -> AnyElement {
    div()
        .absolute()
        .right(px(6.))
        .top(px(5.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .w(px(22.))
        .h(px(22.))
        .rounded_full()
        .bg(rgb(t.row_alt_bg))
        .child(icons::icon(glyph::CARETS_UP_DOWN, 11., t.fg_dim))
        .into_any_element()
}
