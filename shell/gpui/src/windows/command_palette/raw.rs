use gpui::{
    AnyElement, ClickEvent, ClipboardItem, Context, InteractiveElement, IntoElement, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, div, px, rgb,
};

use super::state::{CommandOutput, CommandPalette};
use crate::app::fonts;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::{self, glyph};
use crate::ui::primitives::button;

const SUGGESTIONS: &[&str] = &["status", "log -r @", "diff --stat", "op log"];

pub(super) fn command_view(
    output: Option<&CommandOutput>,
    pending_command: &str,
    history: &[String],
    t: &Theme,
    cx: &mut Context<CommandPalette>,
) -> AnyElement {
    let (cmd_text, hint, hint_color) = match output {
        None | Some(CommandOutput::Idle) => (
            pending_command.to_owned(),
            SharedString::from("Enter"),
            t.fg_faint,
        ),
        Some(CommandOutput::Running { display }) => {
            (display.clone(), SharedString::from("Running..."), t.fg_dim)
        }
        Some(CommandOutput::Done {
            display, exit_code, ..
        }) => {
            let hint = if *exit_code == 0 {
                SharedString::from("exit 0")
            } else {
                SharedString::from(format!("exit {exit_code}"))
            };
            let color = if *exit_code == 0 {
                t.diff_gutter_added_fg
            } else {
                t.diff_gutter_removed_fg
            };
            (display.clone(), hint, color)
        }
    };

    let mut col = div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .py(px(4.))
        .child(command_header(&cmd_text, &hint, hint_color, output, t, cx));

    match output {
        Some(CommandOutput::Done { output, .. }) => {
            col = col
                .child(super::render::divider(t))
                .child(output_pane(output, t));
        }
        Some(CommandOutput::Running { .. }) => {
            col = col.child(discovery(history, t, cx));
        }
        _ => {
            col = col.child(discovery(history, t, cx));
        }
    }

    col.into_any_element()
}

fn command_header(
    cmd: &str,
    hint: &SharedString,
    hint_color: u32,
    output: Option<&CommandOutput>,
    t: &Theme,
    cx: &mut Context<CommandPalette>,
) -> impl IntoElement {
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(10.))
        .px(px(14.))
        .py(px(8.))
        .child(icons::icon(glyph::ARROW_CIRCLE_RIGHT, 14., t.fg_dim))
        .child(
            div()
                .flex_1()
                .font_family(fonts::mono())
                .text_size(ui_font_size(13.))
                .text_color(rgb(t.fg))
                .child(SharedString::from(cmd.to_owned())),
        )
        .child(
            div()
                .text_size(ui_font_size(11.))
                .text_color(rgb(hint_color))
                .child(hint.clone()),
        );

    if let Some(CommandOutput::Done { output, .. }) = output {
        row = row.child(copy_output_button(output.clone(), t, cx));
    }
    row
}

fn discovery(history: &[String], t: &Theme, cx: &mut Context<CommandPalette>) -> impl IntoElement {
    let mut col = div()
        .flex()
        .flex_col()
        .gap(px(10.))
        .px(px(14.))
        .py(px(10.))
        .text_size(ui_font_size(11.))
        .text_color(rgb(t.fg_dim))
        .child("Raw jj commands run in this repository. Use Up/Down for history.")
        .child(suggestion_buttons(t, cx));

    if !history.is_empty() {
        let mut recent = div().flex().flex_col().gap(px(2.)).child(
            div()
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.fg_dim))
                .child("Recent"),
        );
        for command in history.iter().take(5) {
            recent = recent.child(history_row(command, t, cx));
        }
        col = col.child(recent);
    }
    col
}

fn suggestion_buttons(t: &Theme, cx: &mut Context<CommandPalette>) -> impl IntoElement {
    let mut row = div().flex().flex_row().gap(px(8.)).flex_wrap();
    for suggestion in SUGGESTIONS {
        row = row.child(command_chip(format!("jj {suggestion}"), t, cx));
    }
    row
}

fn history_row(command: &str, t: &Theme, cx: &mut Context<CommandPalette>) -> impl IntoElement {
    let query = format!("jj {command}");
    div()
        .id(SharedString::from(format!("command-history-{command}")))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.))
        .px(px(8.))
        .py(px(5.))
        .rounded_sm()
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.selected_bg)))
        .on_click(cx.listener(move |palette, _: &ClickEvent, _, cx| {
            palette.set_query(query.clone(), cx);
        }))
        .child(icons::icon(glyph::ARROW_CLOCKWISE, 13., t.fg_dim))
        .child(
            div()
                .font_family(fonts::mono())
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.fg))
                .child(SharedString::from(format!("jj {command}"))),
        )
}

fn command_chip(
    query: String,
    t: &Theme,
    cx: &mut Context<CommandPalette>,
) -> gpui::Stateful<gpui::Div> {
    let label = query.clone();
    button(
        format!("command-chip-{query}"),
        SharedString::from(label),
        t,
        false,
    )
    .on_click(cx.listener(move |palette, _: &ClickEvent, _, cx| {
        palette.set_query(query.clone(), cx);
    }))
}

fn copy_output_button(
    output: String,
    t: &Theme,
    cx: &mut Context<CommandPalette>,
) -> impl IntoElement {
    button("command-copy-output", "Copy Output", t, false).on_click(cx.listener(
        move |_, _: &ClickEvent, _, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string(output.clone()));
        },
    ))
}

fn output_pane(text: &str, t: &Theme) -> impl IntoElement {
    div()
        .id(SharedString::from("command-output"))
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .px(px(10.))
        .py(px(6.))
        .bg(rgb(t.header_bg))
        .border_1()
        .border_color(rgb(t.border))
        .rounded_sm()
        .font_family(fonts::mono())
        .text_size(ui_font_size(11.))
        .text_color(rgb(t.fg))
        .child(SharedString::from(text.to_owned()))
}
