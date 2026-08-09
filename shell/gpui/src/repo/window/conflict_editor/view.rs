use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, div, px, rgb, rgba,
};
use jayjay_core::MergeHunkSource;
use jayjay_core::external_tools::conflict_marker_count;

use crate::app::fonts;
use crate::app::theme::Theme;
use crate::ui::icons::{glyph, icon};
use crate::ui::merge_editor::{merge_base_toggle, merge_source_panel, merge_source_row};
use crate::ui::primitives::button;

use super::RepoWindow;
use super::hunks::conflict_result_section;

pub(in crate::repo::window) fn conflict_editor_overlay(
    view: &RepoWindow,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let state = &view.conflict_editor;
    let Some(data) = state.data.as_ref() else {
        return div().into_any_element();
    };
    let result_text = state
        .result
        .as_ref()
        .map(|result| result.read(cx).text())
        .unwrap_or_default();
    let source_selected = state
        .selected_source
        .as_ref()
        .is_some_and(|(_, content)| content == &result_text);
    let unresolved = if source_selected {
        0
    } else {
        conflict_marker_count(&result_text, data.marker_length as usize)
    };
    let status = if unresolved == 0 {
        "Resolved".to_owned()
    } else {
        format!("{unresolved} unresolved")
    };
    let save_label = if state.saving {
        "Saving…"
    } else if unresolved > 0 {
        "Save Partial"
    } else {
        "Save Resolution"
    };
    let mut save = button("conflict-editor-save", save_label, t, true)
        .debug_selector(|| "conflict-editor-save".to_owned());
    if data.is_text && !state.saving {
        save = save.on_click(cx.listener(|view, _, _, cx| view.save_conflict_editor(cx)));
    } else {
        save = save.opacity(0.45);
    }

    let panel = div()
        .key_context("ConflictEditor")
        .flex()
        .flex_col()
        .w_full()
        .h_full()
        .max_w(px(1180.))
        .max_h(px(820.))
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
                .child(icon(glyph::GIT_MERGE, 16., t.compare_accent))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .min_w_0()
                        .child(
                            div()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_size(px(14.))
                                .child("Resolve Conflict"),
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
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(rgb(if unresolved == 0 {
                            t.success_fg
                        } else {
                            t.tag_conflict_fg
                        }))
                        .child(status),
                )
                .child(
                    button("conflict-editor-cancel", "Cancel", t, false)
                        .on_click(cx.listener(|view, _, _, cx| view.exit_conflict_editor(cx))),
                )
                .child(save),
        )
        .children(
            (data.side_count == 2)
                .then_some(state.sources.as_ref())
                .flatten()
                .map(|sources| sources_section(view, data, sources, t, cx)),
        )
        .child(conflict_result_section(view, data, &result_text, t, cx));

    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .flex()
        .items_center()
        .justify_center()
        .p(px(16.))
        .bg(rgba(0x00000033))
        .occlude()
        .on_mouse_down(MouseButton::Left, |_: &MouseDownEvent, _, _| {})
        .child(panel)
        .into_any_element()
}

fn sources_section(
    view: &RepoWindow,
    data: &jayjay_core::ConflictEditorData,
    source_views: &[gpui::Entity<crate::ui::text_area::TextArea>; 3],
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let sources = if view.conflict_editor.show_base {
        vec![(MergeHunkSource::Base, "Base", source_views[1].clone())]
    } else {
        vec![
            (MergeHunkSource::Left, "Left", source_views[0].clone()),
            (MergeHunkSource::Right, "Right", source_views[2].clone()),
        ]
    };
    let panels = sources
        .into_iter()
        .enumerate()
        .map(|(index, (source, label, content))| {
            let selector = format!("conflict-editor-use-{}", label.to_lowercase());
            let mut action = button(
                SharedString::from(selector.clone()),
                format!("Use {label}"),
                t,
                false,
            )
            .debug_selector(move || selector.clone());
            if data.is_text {
                action = action.on_click(cx.listener(move |view, _, _, cx| {
                    view.use_conflict_source(source, cx);
                }));
            } else {
                action = action.opacity(0.45);
            }
            merge_source_panel(
                index,
                SharedString::from(format!("conflict-editor-source-scroll-{index}")),
                label,
                content,
                action.into_any_element(),
                t,
            )
        })
        .collect::<Vec<_>>();
    div()
        .flex()
        .flex_col()
        .flex_none()
        .border_t_1()
        .border_color(rgb(t.border))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .h(px(36.))
                .px(px(12.))
                .bg(rgb(t.header_bg))
                .child(
                    div()
                        .text_size(px(12.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child("Sources"),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(rgb(t.fg_dim))
                        .child("Use a complete side as the starting point for the result."),
                )
                .child(div().flex_1())
                .child(
                    merge_base_toggle(
                        "conflict-editor-base-toggle",
                        view.conflict_editor.show_base,
                        t,
                    )
                    .debug_selector(|| "conflict-editor-base-toggle".to_owned())
                    .on_click(cx.listener(|view, _, _, cx| {
                        view.toggle_conflict_base(cx);
                    })),
                ),
        )
        .child(merge_source_row(panels, 260., t))
        .into_any_element()
}
