use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::MergeHunkSource;
use jayjay_core::diff::FileDiff;

use crate::app::actions::{MergeNextHunk, MergePreviousHunk, MergeUseLeftHunk, MergeUseRightHunk};
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::merge_editor::{
    merge_base_toggle, merge_hunk_action_links, merge_hunk_card, merge_hunk_list_container,
    merge_result_mode_button, merge_source_panel, merge_source_row,
};
use crate::ui::primitives::button;

use super::view::{ExternalToolState, ExternalToolWindow};

impl ExternalToolWindow {
    pub(super) fn render_merge(&mut self, t: &Theme, cx: &mut Context<Self>) -> AnyElement {
        let ExternalToolState::Merge {
            session,
            sources,
            result,
        } = &self.state
        else {
            return div().into_any_element();
        };
        let path = session.repo_path.clone();
        let is_text = session.is_text_merge();
        let source_views = if self.show_merge_base {
            vec![(MergeHunkSource::Base, "Base", sources[1].clone())]
        } else {
            vec![
                (MergeHunkSource::Left, "Left", sources[0].clone()),
                (MergeHunkSource::Right, "Right", sources[2].clone()),
            ]
        };
        let result = result.clone();
        let result_text = result.read(cx).text();
        let has_hunks = !session.hunks.is_empty();
        let raw = self.show_merge_raw || !has_hunks;

        let panels = source_views
            .into_iter()
            .enumerate()
            .map(|(index, (source, label, content))| {
                let action = button(
                    SharedString::from(format!("external-use-{label}")),
                    format!("Use {label}"),
                    t,
                    false,
                )
                .on_click(cx.listener(move |view, _, _, cx| {
                    view.use_merge_source(source, cx);
                }))
                .into_any_element();
                merge_source_panel(
                    index,
                    SharedString::from(format!("external-source-scroll-{index}")),
                    label,
                    content,
                    action,
                    t,
                )
            })
            .collect::<Vec<_>>();
        let source_row = merge_source_row(panels, 260., t);
        let result_label = if is_text {
            "Edit freely or replace the result from a source"
        } else {
            "Choose Left, Base, or Right for this non-text conflict"
        };
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .child(
                div()
                    .debug_selector(|| "external-sources".to_owned())
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(t.scaled_control_height(36., 12.)))
                    .px(px(12.))
                    .bg(rgb(t.header_bg))
                    .border_b_1()
                    .border_color(rgb(t.border))
                    .child(
                        div()
                            .text_size(ui_font_size(12.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Sources"),
                    )
                    .child(
                        div()
                            .text_size(ui_font_size(11.))
                            .text_color(rgb(t.fg_dim))
                            .child("Use a complete side as the starting point for the result."),
                    )
                    .child(div().flex_1())
                    .child(
                        merge_base_toggle("external-base-toggle", self.show_merge_base, t)
                            .on_click(cx.listener(|view, _, _, cx| view.toggle_merge_base(cx))),
                    ),
            )
            .child(source_row)
            .child(
                div()
                    .debug_selector(|| "external-result-section".to_owned())
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(t.scaled_control_height(36., 12.)))
                    .px(px(12.))
                    .bg(rgb(t.header_bg))
                    .border_b_1()
                    .border_color(rgb(t.border))
                    .child(
                        div()
                            .text_size(ui_font_size(12.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Result"),
                    )
                    .children(has_hunks.then(|| {
                        merge_result_mode_button("external-result-hunks", "Hunks", !raw, t)
                            .on_click(cx.listener(|view, _, _, cx| {
                                view.set_merge_result_raw(false, cx);
                            }))
                    }))
                    .children(has_hunks.then(|| {
                        merge_result_mode_button("external-result-raw", "Raw", raw, t).on_click(
                            cx.listener(|view, _, _, cx| {
                                view.set_merge_result_raw(true, cx);
                            }),
                        )
                    }))
                    .child(
                        div()
                            .min_w_0()
                            .truncate()
                            .text_size(ui_font_size(11.))
                            .text_color(rgb(t.fg_dim))
                            .child(path),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_size(ui_font_size(11.))
                            .text_color(rgb(t.fg_dim))
                            .child(if raw {
                                result_label
                            } else {
                                "Select a hunk; ⌥← uses Left and ⌥→ uses Right"
                            }),
                    ),
            )
            .child(if raw {
                // Keep the editor mounted while a rehighlight is pending: unmounting the focused TextArea drops keystrokes and routes Escape to the window.
                div()
                    .id("external-result-scroll")
                    .debug_selector(|| "external-result-pane".to_owned())
                    .relative()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .child(result)
                    .into_any_element()
            } else {
                external_hunk_list(&session.hunks, &session.hunk_diffs, &result_text, t, cx)
            })
            .into_any_element()
    }
}

fn external_hunk_list(
    hunks: &[jayjay_core::MergeEditorHunk],
    hunk_diffs: &[FileDiff],
    result: &str,
    t: &Theme,
    cx: &mut Context<ExternalToolWindow>,
) -> AnyElement {
    let cards = hunks
        .iter()
        .zip(hunk_diffs)
        .enumerate()
        .map(|(index, (hunk, unified))| {
            let unresolved = jayjay_core::merge_hunk_is_unresolved(result, hunk);
            let actions = merge_hunk_action_links("external", index, unresolved, t).map(
                |(source, mut action)| {
                    if unresolved {
                        action = action.on_click(cx.listener(move |view, _, _, cx| {
                            view.use_merge_hunk(index, source, cx);
                        }));
                    }
                    action.into_any_element()
                },
            );
            merge_hunk_card(hunk, unified, unresolved, actions, t)
                .on_click(cx.listener(move |view, _, _, cx| view.select_merge_hunk(index, cx)))
        })
        .collect::<Vec<_>>();
    merge_hunk_list_container("external-hunks-scroll", cards)
        .on_action(cx.listener(|view, _: &MergeUseLeftHunk, _, cx| {
            view.use_selected_merge_hunk(MergeHunkSource::Left, cx);
        }))
        .on_action(cx.listener(|view, _: &MergeUseRightHunk, _, cx| {
            view.use_selected_merge_hunk(MergeHunkSource::Right, cx);
        }))
        .on_action(cx.listener(|view, _: &MergePreviousHunk, _, cx| {
            view.move_merge_hunk(-1, cx);
        }))
        .on_action(cx.listener(|view, _: &MergeNextHunk, _, cx| {
            view.move_merge_hunk(1, cx);
        }))
        .into_any_element()
}
