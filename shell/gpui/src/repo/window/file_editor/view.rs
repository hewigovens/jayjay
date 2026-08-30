use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

use crate::app::fonts;
use crate::app::theme::Theme;
use crate::ui::overlay::overlay_layer;
use crate::ui::primitives::button;

use super::super::RepoWindow;

pub(in crate::repo::window) fn file_editor_overlay(
    view: &RepoWindow,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let state = &view.file_editor;
    let Some(data) = state.data.as_ref() else {
        return div().into_any_element();
    };
    let content = state
        .editor
        .as_ref()
        .map(|editor| editor.read(cx).text())
        .unwrap_or_default();
    let has_changes = content != data.content;
    let mut save = button(
        "file-editor-save",
        if state.saving { "Saving…" } else { "Save" },
        t,
        true,
    )
    .debug_selector(|| "file-editor-save".to_owned());
    if has_changes && !state.saving {
        save = save.on_click(cx.listener(|view, _, _, cx| view.save_file_editor(cx)));
    } else {
        save = save.opacity(0.45);
    }

    let panel = div()
        .key_context("FileEditor")
        .flex()
        .flex_col()
        .w_full()
        .h_full()
        .max_w(px(1120.))
        .max_h(px(780.))
        .min_w_0()
        .min_h_0()
        .overflow_hidden()
        .rounded_lg()
        .border_1()
        .border_color(rgb(t.border))
        .bg(rgb(t.detail_bg))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(10.))
                .h(px(52.))
                .px(px(14.))
                .bg(rgb(t.header_bg))
                .border_b_1()
                .border_color(rgb(t.border))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .min_w_0()
                        .child(
                            div()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_size(px(14.))
                                .child("Edit Working-Copy File"),
                        )
                        .child(
                            div()
                                .truncate()
                                .font_family(fonts::mono())
                                .text_size(px(11.))
                                .text_color(rgb(t.fg_dim))
                                .child(data.path.clone()),
                        ),
                )
                .child(div().flex_1())
                .children(has_changes.then(|| {
                    div()
                        .text_size(px(11.))
                        .text_color(rgb(t.tag_conflict_fg))
                        .child("Modified")
                }))
                .child(
                    button("file-editor-cancel", "Cancel", t, false)
                        .on_click(cx.listener(|view, _, _, cx| view.exit_file_editor(cx))),
                )
                .child(save),
        )
        .child(
            div()
                .flex()
                .items_center()
                .h(px(34.))
                .px(px(10.))
                .bg(rgb(t.header_bg))
                .border_b_1()
                .border_color(rgb(t.border))
                .text_size(px(11.))
                .text_color(rgb(t.fg_dim))
                .child("Edits are saved directly to the current working-copy change."),
        )
        .child(
            div()
                .id("file-editor-scroll")
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .children(state.editor.clone()),
        );

    overlay_layer().p(px(16.)).child(panel).into_any_element()
}
