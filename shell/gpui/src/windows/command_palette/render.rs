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

/// Renders the command-mode body: status hint while idle/running, or stdout +
/// stderr panes once the command finishes. `hint` is shown only when `output`
/// is `None` (the user has typed `jj …`/`!…` but not pressed Enter).
pub(super) fn command_view(
    output: Option<&CommandOutput>,
    hint: &str,
    t: &Theme,
) -> impl IntoElement {
    let header = match output {
        None => SharedString::from(hint.to_owned()),
        Some(CommandOutput::Idle) => SharedString::from(hint.to_owned()),
        Some(CommandOutput::Running { display }) => SharedString::from(format!("Running: {display}")),
        Some(CommandOutput::Done { display, success, .. }) => {
            SharedString::from(format!(
                "{} {display}",
                if *success { "✓" } else { "✗" }
            ))
        }
    };
    let header_color = match output {
        Some(CommandOutput::Done { success: false, .. }) => t.diff_gutter_removed_fg,
        Some(CommandOutput::Done { success: true, .. }) => t.diff_gutter_added_fg,
        _ => t.fg_dim,
    };

    let mut col = div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .px(px(14.))
        .py(px(8.))
        .gap(px(8.))
        .child(
            div()
                .text_size(px(11.))
                .text_color(rgb(header_color))
                .child(header),
        );

    if let Some(CommandOutput::Done { stdout, stderr, .. }) = output {
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

    col
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
