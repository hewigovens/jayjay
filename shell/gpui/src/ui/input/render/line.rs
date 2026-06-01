use gpui::{Div, InteractiveElement, ParentElement, SharedString, Styled, div, px, rgb};

use super::selection_bg;
use crate::app::theme::Theme;
use crate::ui::input::{LineEdit, LineInput};

pub fn line_input_content(
    input: &LineInput,
    placeholder: impl Into<SharedString>,
    theme: &Theme,
    caret_id: Option<&'static str>,
) -> Div {
    line_edit_content(
        input.edit(),
        placeholder,
        input.caret_visible(),
        theme,
        caret_id,
    )
}

pub fn line_edit_content(
    input: &LineEdit,
    placeholder: impl Into<SharedString>,
    caret_visible: bool,
    theme: &Theme,
    caret_id: Option<&'static str>,
) -> Div {
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .min_w_0()
        .overflow_hidden();

    if input.is_empty() {
        return row
            .child(caret_slot(caret_visible, theme, caret_id, false))
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .text_color(rgb(theme.fg_faint))
                    .child(placeholder.into()),
            );
    }

    let text = input.text();
    let cursor = input.cursor_offset();
    let selection = input.selection_range();
    if selection.is_empty() {
        let after = &text[cursor..];
        return row
            .child(text_segment(&text[..cursor], theme.fg))
            .child(caret_slot(caret_visible, theme, caret_id, after.is_empty()))
            .child(text_segment(after, theme.fg));
    }

    if input.selection_reversed() {
        row = row
            .child(text_segment(&text[..selection.start], theme.fg))
            .child(caret_slot(caret_visible, theme, caret_id, false))
            .child(selection_segment(&text[selection.clone()], theme))
            .child(text_segment(&text[selection.end..], theme.fg));
    } else {
        let after = &text[selection.end..];
        row = row
            .child(text_segment(&text[..selection.start], theme.fg))
            .child(selection_segment(&text[selection.clone()], theme))
            .child(caret_slot(caret_visible, theme, caret_id, after.is_empty()))
            .child(text_segment(after, theme.fg));
    }
    row
}

fn text_segment(text: &str, color: u32) -> Div {
    div()
        .flex_none()
        .text_color(rgb(color))
        .child(SharedString::from(text.to_owned()))
}

fn selection_segment(text: &str, theme: &Theme) -> Div {
    div()
        .flex_none()
        .bg(selection_bg(theme))
        .text_color(rgb(theme.fg))
        .child(SharedString::from(text.to_owned()))
}

fn caret_slot(visible: bool, theme: &Theme, id: Option<&'static str>, reserve_width: bool) -> Div {
    let mut slot = div()
        .relative()
        .flex()
        .flex_none()
        .items_center()
        .w(px(1.))
        .h(px(16.));
    if !reserve_width {
        slot = slot.mr(-px(1.));
    }
    if visible {
        if let Some(id) = id {
            slot = slot.child(
                div()
                    .id(id)
                    .debug_selector(move || id.to_owned())
                    .absolute()
                    .left(px(0.))
                    .top(px(0.))
                    .w(px(1.))
                    .h(px(16.))
                    .bg(rgb(theme.fg)),
            );
        } else {
            slot = slot.child(
                div()
                    .absolute()
                    .left(px(0.))
                    .top(px(0.))
                    .w(px(1.))
                    .h(px(16.))
                    .bg(rgb(theme.fg)),
            );
        }
    }
    slot
}
