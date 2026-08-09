use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::{ConflictEditorData, MergeHunkSource};

use crate::app::actions::{MergeNextHunk, MergePreviousHunk, MergeUseLeftHunk, MergeUseRightHunk};
use crate::app::theme::Theme;
use crate::ui::merge_editor::{
    merge_hunk_action_links, merge_hunk_card, merge_hunk_list_container, merge_result_mode_button,
};

use super::RepoWindow;

pub(super) fn conflict_result_section(
    view: &RepoWindow,
    data: &ConflictEditorData,
    result_text: &str,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let has_hunks = !data.hunks.is_empty();
    let raw = view.conflict_editor.show_raw || !has_hunks;
    let header = div()
        .flex()
        .items_center()
        .gap(px(6.))
        .h(px(36.))
        .px(px(10.))
        .bg(rgb(t.header_bg))
        .border_b_1()
        .border_color(rgb(t.border))
        .child(
            div()
                .text_size(px(11.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child("Result"),
        )
        .children(has_hunks.then(|| {
            merge_result_mode_button("conflict-result-hunks", "Hunks", !raw, t)
                .on_click(cx.listener(|view, _, _, cx| view.set_conflict_result_raw(false, cx)))
        }))
        .children(has_hunks.then(|| {
            merge_result_mode_button("conflict-result-raw", "Raw", raw, t)
                .on_click(cx.listener(|view, _, _, cx| view.set_conflict_result_raw(true, cx)))
        }))
        .child(div().flex_1())
        .child(
            div()
                .text_size(px(11.))
                .text_color(rgb(t.fg_dim))
                .child(if raw {
                    "Edit markers directly; unresolved markers save as a partial resolution"
                } else {
                    "Select a hunk, then use ⌥← for Left or ⌥→ for Right"
                }),
        );

    let body = if raw {
        // Keep the editor mounted while a rehighlight is pending: unmounting the focused TextArea drops keystrokes and routes Escape to the window.
        div()
            .id("conflict-editor-result-scroll")
            .relative()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .children(view.conflict_editor.result.clone())
            .into_any_element()
    } else {
        hunk_list(view, data, result_text, t, cx)
    };

    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .child(header)
        .child(body)
        .into_any_element()
}

fn hunk_list(
    view: &RepoWindow,
    data: &ConflictEditorData,
    result_text: &str,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let cards = data
        .hunks
        .iter()
        .zip(&view.conflict_editor.hunk_diffs)
        .enumerate()
        .map(|(index, (hunk, unified))| {
            let unresolved = jayjay_core::merge_hunk_is_unresolved(result_text, hunk);
            let actions = merge_hunk_action_links("conflict", index, unresolved, t).map(
                |(source, mut action)| {
                    if unresolved {
                        action = action.on_click(cx.listener(move |view, _, _, cx| {
                            view.use_conflict_hunk(index, source, cx);
                        }));
                    }
                    action.into_any_element()
                },
            );
            merge_hunk_card(hunk, unified, unresolved, actions, t)
                .on_click(cx.listener(move |view, _, _, cx| view.select_conflict_hunk(index, cx)))
        })
        .collect::<Vec<_>>();
    merge_hunk_list_container("conflict-hunks-scroll", cards)
        .on_action(cx.listener(|view, _: &MergeUseLeftHunk, _, cx| {
            view.use_selected_conflict_hunk(MergeHunkSource::Left, cx);
        }))
        .on_action(cx.listener(|view, _: &MergeUseRightHunk, _, cx| {
            view.use_selected_conflict_hunk(MergeHunkSource::Right, cx);
        }))
        .on_action(cx.listener(|view, _: &MergePreviousHunk, _, cx| {
            view.move_conflict_hunk(-1, cx);
        }))
        .on_action(cx.listener(|view, _: &MergeNextHunk, _, cx| {
            view.move_conflict_hunk(1, cx);
        }))
        .into_any_element()
}
