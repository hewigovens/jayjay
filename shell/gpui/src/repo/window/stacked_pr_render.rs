use super::RepoWindow;
use super::stacked_pr::{StackedPrPhase, StackedPrState};
use super::stacked_pr_results::{centered_message, error_body, results_body};
use crate::app::fonts;
use crate::app::theme::Theme;
use crate::ui::icons::{glyph, icon};
use crate::ui::input::line_input_content;
use crate::ui::primitives::{button, icon_label};
use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb, rgba,
};

pub(super) fn stacked_pr_overlay(
    state: &StackedPrState,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let busy = state.ai_in_flight
        || matches!(
            state.phase,
            StackedPrPhase::Loading | StackedPrPhase::Submitting(_)
        );
    let title = if matches!(state.phase, StackedPrPhase::Results(_)) {
        "Stacked PRs Submitted"
    } else {
        "Stacked Pull Requests"
    };
    let panel = div()
        .key_context("StackedPrPanel")
        .flex()
        .flex_col()
        .gap(px(12.))
        .w(px(560.))
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
                .items_center()
                .gap(px(8.))
                .child(
                    icon_label(glyph::GIT_BRANCH, title, 16., t.toggle_active_fg)
                        .text_size(px(14.))
                        .font_weight(gpui::FontWeight::SEMIBOLD),
                )
                .when(busy, |row| {
                    row.child(
                        div()
                            .text_size(px(11.))
                            .text_color(rgb(t.fg_dim))
                            .child("Working…"),
                    )
                }),
        )
        .child(phase_body(state, t, cx));

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
        .occlude()
        .child(panel)
        .into_any_element()
}

fn phase_body(state: &StackedPrState, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    match &state.phase {
        StackedPrPhase::Loading => centered_message("Detecting changes above trunk()…", t),
        StackedPrPhase::Preview(stack) | StackedPrPhase::Submitting(stack) => {
            let submitting = matches!(state.phase, StackedPrPhase::Submitting(_));
            let count = stack.layers.len();
            div()
                .flex()
                .flex_col()
                .gap(px(10.))
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(rgb(t.fg_dim))
                        .child(format!(
                            "{count} change{} — one PR each; bottom targets {}.",
                            if count == 1 { "" } else { "s" },
                            stack.base_bookmark
                        )),
                )
                .child(layer_list(state, t, cx))
                .child(action_row(state, submitting, t, cx))
                .into_any_element()
        }
        StackedPrPhase::Results(result) => results_body(result, t, cx),
        StackedPrPhase::Error(error) => error_body(error, t, cx),
    }
}

fn layer_list(state: &StackedPrState, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    let stack = state.stack().unwrap();
    let mut list = div()
        .id("stacked-pr-layers")
        .flex()
        .flex_col()
        .gap(px(6.))
        .max_h(px(330.))
        .overflow_y_scroll();
    for (index, layer) in stack.layers.iter().enumerate() {
        let base = if index == 0 {
            stack.base_bookmark.as_str()
        } else {
            state.inputs[index - 1].text()
        };
        let warning = state.warning(index);
        let input = line_input_content(
            &state.inputs[index],
            "bookmark name",
            t,
            (state.active_input == Some(index)).then_some("stacked-pr-caret"),
        );
        let mut input_box = div()
            .id(("stacked-pr-input", index))
            .debug_selector(move || format!("stacked-pr-input-{index}"))
            .flex()
            .flex_1()
            .min_w_0()
            .px(px(7.))
            .py(px(4.))
            .rounded_sm()
            .border_1()
            .border_color(rgb(if warning.is_some() {
                t.tag_modified_fg
            } else {
                t.border
            }))
            .font_family(fonts::mono())
            .text_size(px(11.))
            .child(input);
        if matches!(state.phase, StackedPrPhase::Preview(_)) {
            input_box = input_box.cursor_text().on_click(cx.listener(
                move |view, _: &ClickEvent, _, cx| view.activate_stacked_pr_input(index, cx),
            ));
        }
        let mut card = div()
            .id(("stacked-pr-layer", index))
            .debug_selector(move || format!("stacked-pr-layer-{index}"))
            .flex()
            .flex_col()
            .gap(px(5.))
            .px(px(10.))
            .py(px(8.))
            .rounded_md()
            .bg(rgb(t.row_alt_bg))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_size(px(12.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(SharedString::from(layer.title.clone())),
                    )
                    .child(
                        div()
                            .font_family(fonts::mono())
                            .text_size(px(10.))
                            .text_color(rgb(t.fg_dim))
                            .child(SharedString::from(layer.change_id_short.clone())),
                    ),
            )
            .child(input_box)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(5.))
                    .text_size(px(10.))
                    .text_color(rgb(t.fg_dim))
                    .child(icon(glyph::ARROW_RIGHT, 9., t.fg_faint))
                    .child(SharedString::from(base.to_owned())),
            );
        if let Some(warning) = warning {
            card = card.child(
                div()
                    .text_size(px(10.))
                    .text_color(rgb(t.tag_modified_fg))
                    .child(warning),
            );
        }
        list = list.child(card);
    }
    list.into_any_element()
}

fn action_row(
    state: &StackedPrState,
    submitting: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let busy = submitting || state.ai_in_flight;
    let enabled = state.can_submit();
    let submit = if enabled && !busy {
        button("stacked-pr-submit", "Submit", t, true)
            .on_click(cx.listener(|view, _, _, cx| view.submit_stacked_pr(cx)))
            .into_any_element()
    } else {
        button("stacked-pr-submit", "Submit", t, false)
            .opacity(0.45)
            .into_any_element()
    };
    div()
        .flex()
        .justify_end()
        .gap(px(8.))
        .child(if !busy {
            button("stacked-pr-ai-name", "Name with AI", t, false)
                .on_click(cx.listener(|view, _, _, cx| view.generate_stacked_pr_names(cx)))
                .into_any_element()
        } else {
            button(
                "stacked-pr-ai-name",
                if state.ai_in_flight {
                    "Generating…"
                } else {
                    "Name with AI"
                },
                t,
                false,
            )
            .opacity(0.45)
            .into_any_element()
        })
        .child(
            button("stacked-pr-cancel", "Cancel", t, false)
                .when(!busy, |button| {
                    button.on_click(cx.listener(|view, _, _, cx| view.close_stacked_pr(cx)))
                })
                .opacity(if busy { 0.45 } else { 1. }),
        )
        .child(submit)
        .into_any_element()
}
