use super::DescriptionState;
use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, canvas, div, px, rgb, svg,
};
use jayjay_core::ChangeInfo;

use crate::app::fonts;
use crate::app::theme::{FONT_BODY, Theme, ui_font_size};
use crate::repo::window::RepoWindow;
use crate::repo::window::dag_row::first_line;
use crate::ui::icons::{self, glyph, icon};
use crate::ui::primitives::text_tooltip;

const MAXIMUM_HEIGHT: f32 = 80.;

pub(super) fn description_block(
    change: &ChangeInfo,
    state: &DescriptionState,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let title = first_line(&change.description);
    let has_description = !change.description.trim().is_empty();
    let body = change
        .description
        .split_once('\n')
        .map_or("", |(_, body)| body.trim());
    let expanded = state.expanded;
    let can_show_edit_diff = !change.has_conflict && !change.is_empty && !change.is_immutable;
    let view = cx.weak_entity();
    let revision = crate::repo::revset::change_revision(change);
    let content = div()
        .relative()
        .flex()
        .flex_col()
        .flex_shrink_0()
        .gap(px(6.))
        .child(
            div()
                .flex()
                .items_start()
                .gap(px(8.))
                .child(
                    div()
                        .debug_selector(|| "description-title".to_owned())
                        .min_w_0()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_size(ui_font_size(14.))
                        .text_color(rgb(t.fg))
                        .child(SharedString::from(title)),
                )
                .child(edit_button(change, t, cx)),
        )
        .when(!body.is_empty(), |el| {
            el.child(
                div()
                    .debug_selector(|| "description-text".to_owned())
                    .flex_shrink_0()
                    .font_family(fonts::mono())
                    .text_size(ui_font_size(FONT_BODY))
                    .text_color(rgb(t.fg_dim))
                    .child(SharedString::from(body.to_owned())),
            )
        })
        .child(
            canvas(
                move |bounds, _, cx| {
                    let overflows = bounds.size.height > px(MAXIMUM_HEIGHT);
                    let Some(view) = view.upgrade() else { return };
                    if view.read(cx).description.overflows == overflows {
                        return;
                    }
                    let revision = revision.clone();
                    cx.defer(move |cx| {
                        view.update(cx, |view, cx| {
                            if view.description.revision.as_ref() == Some(&revision)
                                && view.description.overflows != overflows
                            {
                                view.description.overflows = overflows;
                                cx.notify();
                            }
                        });
                    });
                },
                |_, _, _, _| {},
            )
            .absolute()
            .size_full(),
        );
    let scroll = div()
        .id(format!(
            "description-body-{}-{expanded}",
            change.commit_id.id
        ))
        .debug_selector(|| "description-body".to_owned())
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .max_h(px(MAXIMUM_HEIGHT * if expanded { 4. } else { 1. }))
        .overflow_y_scroll()
        .child(content);

    let toggle = div()
        .id("description-expansion")
        .size(px(22.))
        .flex_shrink_0()
        .when(state.overflows || expanded, |el| {
            el.debug_selector(|| "description-expansion".to_owned())
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .child(
                    svg()
                        .path(if expanded {
                            icons::COLLAPSE_VERTICAL_SVG
                        } else {
                            icons::EXPAND_VERTICAL_SVG
                        })
                        .size(px(18.))
                        .text_color(rgb(t.fg_dim)),
                )
                .tooltip(text_tooltip(if expanded {
                    "Collapse description"
                } else {
                    "Expand description"
                }))
                .on_click(cx.listener(|view, _, _, cx| {
                    view.description.expanded = !view.description.expanded;
                    cx.notify();
                }))
        });

    div()
        .flex()
        .items_start()
        .gap(px(8.))
        .min_h_0()
        .debug_selector(|| "detail-description".to_owned())
        .when(has_description, |el| el.child(scroll))
        .when(!has_description, |el| {
            el.child(edit_button(change, t, cx))
                .when(change.is_immutable, |el| {
                    el.child(
                        div()
                            .debug_selector(|| "description-empty".to_owned())
                            .text_size(ui_font_size(12.))
                            .text_color(rgb(t.fg_dim))
                            .child("No description"),
                    )
                })
                .child(div().flex_1())
        })
        .when(has_description, |el| el.child(toggle))
        .child(edit_diff_button(can_show_edit_diff, t, cx))
        .into_any_element()
}

fn edit_button(change: &ChangeInfo, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    if change.is_immutable || change.is_working_copy {
        return div().into_any_element();
    }

    let is_empty = change.description.trim().is_empty();
    div()
        .id(SharedString::from("edit-description"))
        .debug_selector(|| "edit-description".to_owned())
        .flex()
        .items_center()
        .justify_center()
        .flex_shrink_0()
        .gap(px(4.))
        .text_size(ui_font_size(12.))
        .text_color(rgb(t.fg_dim))
        .rounded_md()
        .child(icon(glyph::PENCIL, 13., t.fg_dim))
        .child(if is_empty { "Add description" } else { "Edit" })
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .tooltip(text_tooltip(if is_empty {
            "Add description"
        } else {
            "Edit description"
        }))
        .on_click(cx.listener(|view, _, _, cx| view.edit_selected_description(cx)))
        .into_any_element()
}

fn edit_diff_button(visible: bool, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    if !visible {
        return div().into_any_element();
    }

    div()
        .id(SharedString::from("edit-diff"))
        .flex()
        .items_center()
        .justify_center()
        .px(px(9.))
        .h(px(t.scaled_control_height(22., 11.)))
        .rounded_md()
        .bg(rgb(t.toggle_inactive_bg))
        .text_color(rgb(t.toggle_inactive_fg))
        .text_size(ui_font_size(11.))
        .debug_selector(|| "edit-diff".to_owned())
        .cursor_pointer()
        .hover(|s| s.bg(rgb(t.row_alt_bg)))
        .tooltip(text_tooltip("Open Diff Edit Mode"))
        .on_click(cx.listener(|view, _, _, cx| view.enter_diff_edit(cx)))
        .child("Edit Diff...")
        .into_any_element()
}
