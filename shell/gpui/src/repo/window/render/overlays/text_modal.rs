use gpui::{
    AnyElement, Context, CursorStyle, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ParentElement, SharedString, StatefulInteractiveElement, Styled,
    deferred, div, px, rgb, rgba,
};
use jayjay_core::diff::DiffSpanStyle;

use crate::app::fonts::CodeText as _;
use crate::app::theme::{Theme, with_alpha};
use crate::repo::window::commit_ai::generate_button;
use crate::repo::window::note_composer::NoteContextLine;
use crate::repo::window::{RepoWindow, TextModalAction};
use crate::ui::overlay::{PromptSlots, PromptStyle};
use crate::ui::primitives::{HelpTag, button, checkbox_row};

pub(crate) fn text_modal_overlay(
    view: &RepoWindow,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> Option<AnyElement> {
    let modal = view.text_modal()?;
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
    let committing = view.vm.read(cx).committing;
    let working_copy = matches!(
        modal.action,
        TextModalAction::EditDescription {
            working_copy: true,
            ..
        }
    );
    let leading_actions = if working_copy {
        working_copy_actions(view, committing, t, cx)
    } else {
        Vec::new()
    };
    let resize_grip = modal.prompt.is_resizable().then(|| resize_grip(t, cx));
    let resizing = modal.prompt.is_resizing();
    let overlay = modal.prompt.overlay(
        &PromptStyle {
            // Composer's own key context: mod+Return saves only while this overlay's input has focus, never the commit box or other text modals, which don't set `context` so never wrap in it.
            key_context: modal.context.is_some().then_some("NoteComposer"),
            primary_enabled: !(working_copy && committing),
            ..PromptStyle::new(520., "text-modal-cancel", "text-modal-primary")
        },
        t,
        cx,
        PromptSlots::new(before_input, after_input)
            .with_leading_actions(leading_actions)
            .with_corner(resize_grip),
        |view, cx| view.close_text_modal(cx),
        |view, cx| view.submit_text_modal(cx),
    );
    if !modal.prompt.is_resizable() {
        return Some(overlay);
    }
    // Always mounted, so a release that lands before the next frame still ends the drag.
    let mut tracker = div()
        .absolute()
        .inset_0()
        .on_mouse_move(cx.listener(|view, ev: &MouseMoveEvent, window, cx| {
            if ev.pressed_button == Some(MouseButton::Left) {
                view.resize_text_modal(ev.position, window.viewport_size(), cx);
            } else {
                view.end_text_modal_resize(cx);
            }
        }))
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(|view, _: &MouseUpEvent, _, cx| view.end_text_modal_resize(cx)),
        );
    if resizing {
        tracker = tracker.cursor(CursorStyle::ResizeUpLeftDownRight);
    }
    Some(
        div()
            .absolute()
            .inset_0()
            .child(overlay)
            .child(deferred(tracker))
            .into_any_element(),
    )
}

fn resize_grip(t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    div()
        .absolute()
        .right_0()
        .bottom_0()
        .size(px(18.))
        .cursor(CursorStyle::ResizeUpLeftDownRight)
        .debug_selector(|| "text-modal-resize".to_owned())
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|view, ev: &MouseDownEvent, _, cx| {
                cx.stop_propagation();
                view.start_text_modal_resize(ev.position, cx);
            }),
        )
        .child(
            div()
                .absolute()
                .right(px(5.))
                .bottom(px(5.))
                .size(px(7.))
                .border_r_2()
                .border_b_2()
                .rounded_br(px(3.))
                .border_color(rgb(t.fg_dim)),
        )
        .into_any_element()
}

fn working_copy_actions(
    view: &RepoWindow,
    committing: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> Vec<AnyElement> {
    let commit = button("text-modal-commit", jayjay_i18n::repo::commit(), t, false)
        .debug_selector(|| "text-modal-commit".to_owned())
        .help(jayjay_i18n::repo::describe_start_new_change_jj_commit());
    let commit = if committing {
        commit.opacity(0.45)
    } else {
        commit.on_click(cx.listener(|view, _, _, cx| view.commit_from_description_sheet(cx)))
    };
    vec![
        generate_button(view, RepoWindow::generate_description_sheet_message, t, cx),
        commit.into_any_element(),
    ]
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
                .code_text(11., t)
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
        .code_text(11., t)
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
