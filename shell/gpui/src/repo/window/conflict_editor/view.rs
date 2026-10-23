use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::MergeHunkSource;
use jayjay_core::external_tools::conflict_marker_count;

use crate::app::fonts::CodeText as _;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::{glyph, icon};
use crate::ui::merge_editor::{merge_base_toggle, merge_source_panel, merge_source_row};
use crate::ui::overlay::overlay_layer;
use crate::ui::primitives::button;

use super::RepoWindow;
use super::hunks::conflict_result_section;

pub(crate) fn conflict_editor_overlay(
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
        jayjay_i18n::repo::resolved()
    } else {
        jayjay_i18n::merge::unresolved_count(unresolved)
    };
    let save_label = if state.saving {
        jayjay_i18n::repo::saving_ellipsis()
    } else if unresolved > 0 {
        jayjay_i18n::repo::save_partial()
    } else {
        jayjay_i18n::repo::save_resolution()
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
                .h(px(t.scaled_control_height(52., 14.)))
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
                                .text_size(ui_font_size(14.))
                                .child(jayjay_i18n::repo::resolve_conflict()),
                        )
                        .child(
                            div()
                                .truncate()
                                .code_text(11., t)
                                .text_color(rgb(t.fg_dim))
                                .child(data.path.clone()),
                        ),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .text_size(ui_font_size(11.))
                        .text_color(rgb(if unresolved == 0 {
                            t.success_fg
                        } else {
                            t.tag_conflict_fg
                        }))
                        .child(status),
                )
                .child(
                    button(
                        "conflict-editor-cancel",
                        jayjay_i18n::common::cancel(),
                        t,
                        false,
                    )
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

    overlay_layer().p(px(16.)).child(panel).into_any_element()
}

fn sources_section(
    view: &RepoWindow,
    data: &jayjay_core::ConflictEditorData,
    source_views: &[gpui::Entity<crate::ui::text_area::TextArea>; 3],
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let sources = if view.conflict_editor.show_base {
        vec![(
            MergeHunkSource::Base,
            jayjay_i18n::repo::base(),
            source_views[1].clone(),
        )]
    } else {
        vec![
            (
                MergeHunkSource::Left,
                jayjay_i18n::repo::left(),
                source_views[0].clone(),
            ),
            (
                MergeHunkSource::Right,
                jayjay_i18n::repo::right(),
                source_views[2].clone(),
            ),
        ]
    };
    let panels = sources
        .into_iter()
        .enumerate()
        .map(|(index, (source, label, content))| {
            let selector = format!("conflict-editor-use-{source:?}").to_lowercase();
            let mut action = button(
                SharedString::from(selector.clone()),
                jayjay_i18n::repo::use_side(&label),
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
                .h(px(t.scaled_control_height(36., 12.)))
                .px(px(12.))
                .bg(rgb(t.header_bg))
                .child(
                    div()
                        .text_size(ui_font_size(12.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(jayjay_i18n::repo::sources()),
                )
                .child(
                    div()
                        .text_size(ui_font_size(11.))
                        .text_color(rgb(t.fg_dim))
                        .child(jayjay_i18n::repo::use_a_complete_side_as_the()),
                )
                .child(div().flex_1())
                .child(
                    merge_base_toggle(
                        "conflict-editor-base-toggle",
                        view.conflict_editor.show_base,
                        t,
                    )
                    .debug_selector(|| "conflict-editor-base-toggle".to_owned())
                    .on_click(cx.listener(|view, _, window, cx| {
                        view.toggle_conflict_base(window, cx);
                    })),
                ),
        )
        .child(merge_source_row(panels, 260., t))
        .into_any_element()
}
