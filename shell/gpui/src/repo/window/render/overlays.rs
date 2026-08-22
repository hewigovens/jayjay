use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, div, px, rgb, rgba,
};
use jayjay_core::diff::DiffSpanStyle;

use crate::app::theme::{Theme, with_alpha};
use crate::repo::window::note_composer::NoteContextLine;
use crate::repo::window::{RepoWindow, TextModalState};
use crate::ui::icons::glyph;
use crate::ui::primitives::{button, checkbox_row, icon_label};

pub(super) fn text_modal_overlay(
    modal: &TextModalState,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let mut panel = div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .w(px(520.))
        .max_w_full()
        .px(px(18.))
        .py(px(16.))
        .rounded_lg()
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(t.header_bg))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .child(
                    icon_label(glyph::PENCIL_CIRCLE, modal.title.clone(), 16., t.fg_dim)
                        .text_size(px(14.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(t.fg)),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .font_family(crate::app::fonts::mono())
                        .text_size(px(11.))
                        .text_color(rgb(t.fg_dim))
                        .child(modal.subtitle.clone()),
                ),
        );
    if let Some(context) = modal.context.as_ref() {
        // Composer's own key context: mod+Return saves only while this overlay's input has focus, never the commit box or other text modals, which don't set `context` so never wrap in it.
        panel = panel.key_context("NoteComposer");
        if !context.lines.is_empty() {
            panel = panel.child(note_context_preview(&context.lines, &context.input, t));
        }
    }
    panel = panel.child(modal.input.clone());
    if let Some(checkbox) = modal.checkbox.as_ref() {
        panel = panel.child(
            checkbox_row(
                "text-modal-checkbox",
                checkbox.label.clone(),
                checkbox.checked,
                t,
            )
            .on_click(cx.listener(|view, _, _, cx| {
                view.toggle_text_modal_checkbox(cx);
            })),
        );
    }
    if let Some(paths) = modal.file_list.as_ref() {
        panel = panel.child(file_list_preview(paths, t));
    }
    panel = panel.child(
        div()
            .flex()
            .flex_row()
            .justify_end()
            .gap(px(8.))
            .child(
                button("text-modal-cancel", "Cancel", t, false).on_click(cx.listener(
                    |view, _, _, cx| {
                        view.close_text_modal(cx);
                    },
                )),
            )
            .child(
                button("text-modal-primary", modal.primary_label.clone(), t, true).on_click(
                    cx.listener(|view, _, _, cx| {
                        view.submit_text_modal(cx);
                    }),
                ),
            ),
    );

    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0x00000033))
        // occlude() also swallows scroll-wheel events, or scrolling the modal's file list scrolls the diff underneath.
        .occlude()
        .on_mouse_down(MouseButton::Left, |_: &MouseDownEvent, _, _| {})
        .child(panel)
        .into_any_element()
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
                t.file_modified_color,
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
                .text_size(px(11.))
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
        .text_size(px(11.))
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

pub(super) fn error_overlay(
    message: gpui::SharedString,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0x00000033))
        .on_mouse_down(MouseButton::Left, |_: &MouseDownEvent, _, _| {})
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(12.))
                .w(px(460.))
                .max_w_full()
                .px(px(20.))
                .py(px(18.))
                .rounded_lg()
                .border_1()
                .border_color(rgb(t.border))
                .bg(rgb(t.header_bg))
                .child(
                    div().flex().flex_row().items_center().child(
                        icon_label(glyph::WARNING, "Operation failed", 18., t.error_fg)
                            .text_size(px(14.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(t.fg)),
                    ),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .line_height(px(18.))
                        .text_color(rgb(t.fg_dim))
                        .child(message),
                )
                .child(
                    div().flex().flex_row().justify_end().child(
                        div()
                            .id("error-ok")
                            .px(px(12.))
                            .py(px(5.))
                            .rounded_md()
                            .bg(rgb(t.toggle_active_bg))
                            .text_color(rgb(t.toggle_active_fg))
                            .text_size(px(12.))
                            .cursor_pointer()
                            .on_click(cx.listener(|view, _, _, cx| {
                                view.vm.update(cx, |vm, cx| {
                                    vm.clear_error();
                                    cx.notify();
                                });
                            }))
                            .child("OK"),
                    ),
                ),
        )
        .into_any_element()
}

pub(super) fn toast_overlay(message: gpui::SharedString, t: &Theme) -> AnyElement {
    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .flex()
        .items_center()
        .justify_center()
        .px(px(24.))
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .max_w(px(520.))
                .px(px(18.))
                .py(px(10.))
                .rounded(px(14.))
                .border_1()
                .border_color(rgb(t.border))
                .bg(rgb(t.header_bg))
                .text_size(px(13.))
                .line_height(px(18.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(rgb(t.fg))
                .child(message),
        )
        .into_any_element()
}
