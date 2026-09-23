use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb, rgba,
};
use jayjay_core::diff::DiffSpanStyle;

use crate::app::theme::{Theme, ui_font_size, with_alpha};
use crate::repo::window::note_composer::NoteContextLine;
use crate::repo::window::{RepoWindow, TextModalState};
use crate::ui::overlay::{PromptSlots, PromptStyle};
use crate::ui::primitives::checkbox_row;

pub(crate) fn text_modal_overlay(
    modal: &TextModalState,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let before_input = modal.context.as_ref().and_then(|context| {
        (!context.lines.is_empty()).then(|| note_context_preview(&context.lines, &context.input, t))
    });
    let mut after_input = Vec::new();
    if let Some(checkbox) = modal.checkbox.as_ref() {
        after_input.push(
            checkbox_row(
                "text-modal-checkbox",
                checkbox.label.clone(),
                checkbox.checked,
                t,
            )
            .on_click(cx.listener(|view, _, _, cx| {
                view.toggle_text_modal_checkbox(cx);
            }))
            .into_any_element(),
        );
    }
    if let Some(paths) = modal.file_list.as_ref() {
        after_input.push(file_list_preview(paths, t));
    }
    modal.prompt.overlay(
        &PromptStyle {
            // Composer's own key context: mod+Return saves only while this overlay's input has focus, never the commit box or other text modals, which don't set `context` so never wrap in it.
            key_context: modal.context.is_some().then_some("NoteComposer"),
            ..PromptStyle::new(520., "text-modal-cancel", "text-modal-primary")
        },
        t,
        cx,
        PromptSlots::new(before_input, after_input),
        |view, cx| view.close_text_modal(cx),
        |view, cx| view.submit_text_modal(cx),
    )
}

fn note_context_preview(
    lines: &[NoteContextLine],
    context_input: &gpui::Entity<crate::ui::text_area::TextArea>,
    t: &Theme,
) -> AnyElement {
    const ROW_HEIGHT: f32 = 22.;
    let mut backgrounds = div().flex().flex_col();
    for line in lines {
        let marker = match line.style {
            DiffSpanStyle::Added => "+",
            DiffSpanStyle::Removed => "-",
            _ => " ",
        };
        let bg = if line.is_anchor {
            rgba(with_alpha(
                t.note_accent,
                if t.is_dark { 0x2a } else { 0x22 },
            ))
        } else {
            match line.style {
                DiffSpanStyle::Added => rgb(t.diff_added_bg),
                DiffSpanStyle::Removed => rgb(t.diff_removed_bg),
                _ => rgb(t.diff_context_bg),
            }
        };
        backgrounds = backgrounds.child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .h(px(ROW_HEIGHT))
                .px(px(8.))
                .bg(bg)
                .font_family(crate::app::fonts::mono())
                .text_size(ui_font_size(11.))
                .child(
                    div()
                        .flex_none()
                        .w(px(12.))
                        .text_color(rgb(t.fg_dim))
                        .child(SharedString::from(marker)),
                ),
        );
    }

    div()
        .relative()
        .w_full()
        .h(px(ROW_HEIGHT * lines.len() as f32))
        .rounded_md()
        .overflow_hidden()
        .border_1()
        .border_color(rgb(t.border))
        .child(backgrounds)
        .child(
            div()
                .absolute()
                .top_0()
                .bottom_0()
                .left(px(24.))
                .right(px(8.))
                .debug_selector(|| "review-note-selectable-code".to_owned())
                .child(context_input.clone()),
        )
        .into_any_element()
}

/// SwiftUI parity (`SplitSheetView.fileList`): plain monospace path rows, sorted, scrolling past 10 entries.
fn file_list_preview(paths: &[SharedString], t: &Theme) -> AnyElement {
    const MAX_VISIBLE: usize = 10;
    let mut list = div()
        .id("text-modal-file-list")
        .flex()
        .flex_col()
        .w_full()
        .gap(px(2.))
        .font_family(crate::app::fonts::mono())
        .text_size(ui_font_size(11.))
        .text_color(rgb(t.fg_dim));
    for path in paths {
        // Truncation needs the flex_1/min_w_0 inner cell, or scrolled rows collapse to bare ellipses.
        list = list.child(
            div()
                .flex()
                .flex_row()
                .w_full()
                .min_w_0()
                .child(div().flex_1().min_w_0().truncate().child(path.clone())),
        );
    }
    if paths.len() > MAX_VISIBLE {
        list = list.h(px(150.)).overflow_y_scroll();
    }
    list.into_any_element()
}
