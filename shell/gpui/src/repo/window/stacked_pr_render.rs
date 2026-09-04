use super::RepoWindow;
use super::stacked_pr::{StackedPrPhase, StackedPrState};
use super::stacked_pr_layers::layer_list;
use super::stacked_pr_results::{centered_message, error_body, results_body};
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons::{glyph, icon};
use crate::ui::overlay::overlay_layer;
use crate::ui::primitives::{button, icon_label};
use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use jayjay_core::Stack;

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
        .key_context(if state.active_input.is_some() {
            "StackedPrPanel StackedPrInput"
        } else {
            "StackedPrPanel"
        })
        .flex()
        .flex_col()
        .gap(px(14.))
        .w(px(480.))
        .max_w_full()
        .p(px(20.))
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
                        .text_size(ui_font_size(15.))
                        .font_weight(gpui::FontWeight::SEMIBOLD),
                )
                .when(busy, |row| {
                    row.child(
                        div()
                            .text_size(ui_font_size(11.))
                            .text_color(rgb(t.fg_dim))
                            .child("Working…"),
                    )
                }),
        )
        .child(phase_body(state, t, cx));

    overlay_layer().child(panel).into_any_element()
}

fn phase_body(state: &StackedPrState, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    match &state.phase {
        StackedPrPhase::Loading => centered_message("Detecting changes above trunk()…", t),
        StackedPrPhase::Preview(stack) | StackedPrPhase::Submitting(stack) => {
            let submitting = matches!(state.phase, StackedPrPhase::Submitting(_));
            div()
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(subtitle_row(state, stack, submitting, t, cx))
                .child(layer_list(state, stack, t, cx))
                .child(action_row(state, submitting, t, cx))
                .into_any_element()
        }
        StackedPrPhase::Results(result) => results_body(result, t, cx),
        StackedPrPhase::Error(error) => error_body(error, t, cx),
    }
}

fn subtitle_row(
    state: &StackedPrState,
    stack: &Stack,
    submitting: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let count = stack.layers.len();
    let busy = submitting || state.ai_in_flight;
    let label = if state.ai_in_flight {
        "Generating…"
    } else {
        "Generate bookmarks"
    };
    let mut generate = button("stacked-pr-ai-name", label, t, false)
        .text_size(ui_font_size(11.))
        .opacity(if busy { 0.45 } else { 1. });
    if !busy {
        generate =
            generate.on_click(cx.listener(|view, _, _, cx| view.generate_stacked_pr_names(cx)));
    }
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(ui_font_size(12.))
                .text_color(rgb(t.fg_dim))
                .child(format!(
                    "{count} change{} — one PR each, bottom targets {}.",
                    if count == 1 { "" } else { "s" },
                    stack.base_bookmark
                )),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(4.))
                .child(icon(glyph::SPARKLE, 11., t.fg_dim))
                .child(generate),
        )
        .into_any_element()
}

fn action_row(
    state: &StackedPrState,
    submitting: bool,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let busy = submitting || state.ai_in_flight;
    let enabled = state.can_submit();
    let submit = if enabled {
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
        .justify_center()
        .gap(px(12.))
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
