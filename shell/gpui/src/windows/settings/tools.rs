use jayjay_core::{CliStatus, check_gh_environment, check_glab_environment, check_jj_environment};

use crate::app::config::AppConfig;
use crate::app::tools::{EDITOR_OPTIONS, TERMINAL_OPTIONS};
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, SharedString, Styled, div, px, rgb,
};

use super::SettingsView;
use super::shared::{current_value, field_row, section_title};
use crate::app::theme::Theme;
use crate::ui::icons::{self, glyph};

pub(super) fn tools_section(
    cfg: &AppConfig,
    t: &Theme,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(section_title("Tools", t))
        .child(field_row(
            "External editor",
            dropdown_button("editor", EDITOR_OPTIONS, &cfg.tools.external_editor, t, cx),
            "Used by 'Open in Editor' actions.",
            t,
        ))
        .child(field_row(
            "Custom editor command",
            current_value(
                if cfg.tools.custom_editor_command.is_empty() {
                    "(none)"
                } else {
                    cfg.tools.custom_editor_command.as_str()
                },
                t,
            ),
            "Required when editor = 'Custom'. Edit ~/.config/jayjay/config.toml.",
            t,
        ))
        .child(field_row(
            "Terminal",
            dropdown_button("terminal", TERMINAL_OPTIONS, &cfg.tools.terminal, t, cx),
            "Used by 'Open in Terminal'.",
            t,
        ))
        .child(cli_tools(t))
        .into_any_element()
}

/// CLI tool availability, mirroring the SwiftUI Tools status rows.
fn cli_tools(t: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .max_w(px(360.))
        .gap(px(2.))
        .child(
            div()
                .pb(px(4.))
                .text_size(px(11.))
                .text_color(rgb(t.fg_faint))
                .child("Command-line tools"),
        )
        .child(cli_row("jj", glyph::GIT_BRANCH, check_jj_environment(), t))
        .child(cli_row("gh", glyph::GIT_MERGE, check_gh_environment(), t))
        .child(cli_row("glab", glyph::GIT_MERGE, check_glab_environment(), t))
}

fn cli_row(
    name: &'static str,
    glyph_str: &'static str,
    status: CliStatus,
    t: &Theme,
) -> impl IntoElement {
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .py(px(5.))
        .px(px(8.))
        .rounded_sm()
        .child(icons::icon(glyph_str, 14., t.fg_dim))
        .child(
            div()
                .w(px(48.))
                .text_size(px(12.))
                .text_color(rgb(t.fg))
                .child(name),
        )
        .child(div().flex_1());

    if status.is_installed {
        let detail = if status.version.is_empty() {
            status.path.clone()
        } else {
            format!("{name} {}", status.version)
        };
        row = row
            .child(
                div()
                    .font_family(crate::app::fonts::mono())
                    .text_size(px(11.))
                    .text_color(rgb(t.fg_dim))
                    .child(SharedString::from(detail)),
            )
            .child(icons::icon(glyph::CHECK, 13., t.tag_added_fg));
    } else {
        row = row
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(rgb(t.fg_faint))
                    .child("Not installed"),
            )
            .child(icons::icon(glyph::X_CIRCLE, 13., t.fg_faint));
    }

    row
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
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .min_w(px(180.))
        .justify_between()
        .px(px(10.))
        .py(px(4.))
        .rounded_sm()
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(t.toggle_inactive_bg))
        .text_size(px(11.))
        .text_color(rgb(t.fg))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |view, ev: &MouseDownEvent, _w, cx| {
                view.open_dropdown(SharedString::from(field_id), ev.position, cx);
            }),
        )
        .child(SharedString::from(label.to_owned()))
        .child(icons::icon(glyph::CARET_DOWN, 10., t.fg_faint))
        .into_any_element()
}
