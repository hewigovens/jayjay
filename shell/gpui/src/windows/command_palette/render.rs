use gpui::{AnyElement, IntoElement, ParentElement, SharedString, Styled, div, px, rgb};

use super::CommandOutput;
use super::actions::{ACTIONS, PaletteAction};
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::ui::icons::{self, glyph};

pub(super) fn query_box(query: &str, t: &Theme) -> impl IntoElement {
    let display = if query.is_empty() {
        SharedString::from("Type to search…")
    } else {
        SharedString::from(query.to_owned())
    };
    let color = if query.is_empty() { t.fg_faint } else { t.fg };
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(14.))
        .py(px(10.))
        .text_size(px(14.))
        .text_color(rgb(color))
        .child(icons::icon(glyph::SEARCH, 14., t.fg_dim))
        .child(display)
}

pub(super) fn divider(t: &Theme) -> impl IntoElement {
    div().h(px(1.)).w_full().bg(rgb(t.border))
}

pub(super) fn action_list(visible: &[usize], selected: usize, t: &Theme) -> AnyElement {
    if visible.is_empty() {
        return div()
            .flex()
            .flex_1()
            .items_center()
            .justify_center()
            .text_color(rgb(t.fg_dim))
            .child("No matches")
            .into_any_element();
    }
    let mut col = div().flex().flex_col().flex_1().min_h_0().py(px(4.));
    for (vis_ix, action_ix) in visible.iter().enumerate() {
        let action = &ACTIONS[*action_ix];
        col = col.child(action_row(action, vis_ix == selected, t));
    }
    col.into_any_element()
}

/// Renders the command-mode body. The full command is always shown as a
/// `$ <cmd>` row in monospace at the top so the user sees exactly what will
/// run (or did run); status/output panes follow.
pub(super) fn command_view(
    output: Option<&CommandOutput>,
    hint_command: &str,
    t: &Theme,
) -> impl IntoElement {
    let (cmd_text, status) = match output {
        None | Some(CommandOutput::Idle) => (hint_command.to_owned(), CommandStatus::Pending),
        Some(CommandOutput::Running { display }) => (display.clone(), CommandStatus::Running),
        Some(CommandOutput::Done {
            display, success, ..
        }) => (
            display.clone(),
            if *success {
                CommandStatus::Ok
            } else {
                CommandStatus::Err
            },
        ),
    };

    let mut col = div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .px(px(14.))
        .py(px(8.))
        .gap(px(8.))
        .child(command_line(&cmd_text, status, t));

    match output {
        None | Some(CommandOutput::Idle) => {
            col = col.child(
                div()
                    .text_size(px(10.))
                    .text_color(rgb(t.fg_faint))
                    .child("Press Enter to run · Esc to cancel"),
            );
        }
        Some(CommandOutput::Running { .. }) => {
            col = col.child(
                div()
                    .text_size(px(10.))
                    .text_color(rgb(t.fg_dim))
                    .child("Running…"),
            );
        }
        Some(CommandOutput::Done { stdout, stderr, .. }) => {
            if !stdout.is_empty() {
                col = col.child(output_pane(stdout, t.fg, t));
            }
            if !stderr.is_empty() {
                col = col.child(output_pane(stderr, t.diff_gutter_removed_fg, t));
            }
            if stdout.is_empty() && stderr.is_empty() {
                col = col.child(
                    div()
                        .text_size(px(11.))
                        .text_color(rgb(t.fg_faint))
                        .child("(no output)"),
                );
            }
            col = col.child(
                div()
                    .text_size(px(10.))
                    .text_color(rgb(t.fg_faint))
                    .child("Esc to dismiss · run another command from the query"),
            );
        }
    }

    col
}

#[derive(Clone, Copy)]
enum CommandStatus {
    Pending,
    Running,
    Ok,
    Err,
}

fn command_line(cmd: &str, status: CommandStatus, t: &Theme) -> impl IntoElement {
    let (marker, marker_color) = match status {
        CommandStatus::Pending => ("$", t.fg_dim),
        CommandStatus::Running => ("…", t.fg_dim),
        CommandStatus::Ok => ("✓", t.diff_gutter_added_fg),
        CommandStatus::Err => ("✗", t.diff_gutter_removed_fg),
    };
    div()
        .flex()
        .flex_row()
        .gap(px(8.))
        .px(px(10.))
        .py(px(6.))
        .bg(rgb(t.header_bg))
        .border_1()
        .border_color(rgb(t.border))
        .rounded_sm()
        .font_family(fonts::mono())
        .text_size(px(12.))
        .child(
            div()
                .flex_none()
                .w(px(14.))
                .text_color(rgb(marker_color))
                .child(marker),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_color(rgb(t.fg))
                .child(SharedString::from(cmd.to_owned())),
        )
}

fn output_pane(text: &str, fg: u32, t: &Theme) -> impl IntoElement {
    div()
        .flex_1()
        .min_h_0()
        .px(px(10.))
        .py(px(6.))
        .bg(rgb(t.header_bg))
        .border_1()
        .border_color(rgb(t.border))
        .rounded_sm()
        .font_family(fonts::mono())
        .text_size(px(11.))
        .text_color(rgb(fg))
        .child(SharedString::from(text.to_owned()))
}

fn action_row(action: &'static PaletteAction, is_selected: bool, t: &Theme) -> impl IntoElement {
    let (bg, fg, glyph_color) = if is_selected {
        (t.selected_bg, t.fg, t.toggle_active_fg)
    } else {
        (t.detail_bg, t.fg, t.fg_dim)
    };
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(10.))
        .px(px(14.))
        .py(px(7.))
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .text_size(px(13.))
        .child(icons::icon(action.glyph_str, 14., glyph_color))
        .child(SharedString::from(action.name))
}
