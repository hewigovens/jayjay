use super::RepoWindow;
use super::stacked_pr::{StackedPrPhase, StackedPrState};
use crate::app::fonts;
use crate::app::theme::{Theme, with_alpha};
use crate::ui::icons::{glyph, icon};
use crate::ui::input::line_input_content;
use gpui::{
    AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb, rgba,
};
use jayjay_core::{Stack, StackLayer};

pub(super) fn layer_list(
    state: &StackedPrState,
    stack: &Stack,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let mut list = div()
        .id("stacked-pr-layers")
        .flex()
        .flex_col()
        .gap(px(6.))
        .max_h(px(280.))
        .overflow_y_scroll();
    // Top of the stack first (core layers are bottom→top), matching the SwiftUI panel.
    for (index, layer) in stack.layers.iter().enumerate().rev() {
        list = list.child(layer_card(state, stack, index, layer, t, cx));
    }
    list.into_any_element()
}

fn layer_card(
    state: &StackedPrState,
    stack: &Stack,
    index: usize,
    layer: &StackLayer,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let base = if index == 0 {
        stack.base_bookmark.as_str()
    } else {
        state.inputs[index - 1].text()
    };
    let mut base_row = div()
        .flex()
        .items_center()
        .gap(px(5.))
        .font_family(fonts::mono())
        .text_size(px(10.))
        .text_color(rgb(t.fg_faint))
        .child(icon(glyph::ARROW_RIGHT, 8., t.fg_faint))
        .child(SharedString::from(base.to_owned()));
    if !layer.bookmark_existed {
        base_row = base_row.child(
            div()
                .px(px(4.))
                .rounded_full()
                .bg(rgba(with_alpha(t.file_added_color, 0x2e)))
                .text_size(px(9.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(t.file_added_color))
                .child("new"),
        );
    }
    let mut column = div().flex().flex_col().flex_1().min_w_0().gap(px(4.));
    if !layer.title.is_empty() {
        column = column.child(
            div()
                .truncate()
                .text_size(px(12.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(SharedString::from(layer.title.clone())),
        );
    }
    column = column
        .child(bookmark_field(state, index, t, cx))
        .child(base_row);
    div()
        .id(("stacked-pr-layer", index))
        .debug_selector(move || format!("stacked-pr-layer-{index}"))
        .flex()
        .items_start()
        .gap(px(10.))
        .px(px(10.))
        .py(px(7.))
        .rounded_lg()
        .bg(rgba(with_alpha(t.fg, if t.is_dark { 0x10 } else { 0x0a })))
        .child(icon(glyph::GIT_BRANCH, 11., t.fg_faint))
        .child(column)
        .into_any_element()
}

fn bookmark_field(
    state: &StackedPrState,
    index: usize,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let editing = state.active_input == Some(index);
    let editable = matches!(state.phase, StackedPrPhase::Preview(_));
    let warning = state.warning(index);
    let warning_icon = warning.map(|message| {
        div()
            .id(("stacked-pr-warning", index))
            .flex_none()
            .child(icon(glyph::WARNING, 10., t.tag_modified_fg))
            .child(
                div()
                    .text_size(px(10.))
                    .text_color(rgb(t.tag_modified_fg))
                    .child(message),
            )
            .flex()
            .items_center()
            .gap(px(4.))
    });
    if editing {
        let input = line_input_content(
            &state.inputs[index],
            "bookmark name",
            t,
            Some("stacked-pr-caret"),
        );
        return div()
            .flex()
            .items_center()
            .gap(px(5.))
            .child(
                div()
                    .id(("stacked-pr-input", index))
                    .debug_selector(move || format!("stacked-pr-input-{index}"))
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .h(px(24.))
                    .items_center()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .px(px(7.))
                    .rounded_sm()
                    .border_1()
                    .border_color(rgb(if warning.is_some() {
                        t.tag_modified_fg
                    } else {
                        t.border
                    }))
                    .font_family(fonts::mono())
                    .text_size(px(11.))
                    .cursor_text()
                    .child(input),
            )
            .children(warning_icon)
            .child(
                div()
                    .id(("stacked-pr-done", index))
                    .cursor_pointer()
                    .child(icon(glyph::CHECK, 12., t.file_added_color))
                    .on_click(cx.listener(|view, _: &ClickEvent, _, cx| {
                        view.deactivate_stacked_pr_input(cx)
                    })),
            )
            .into_any_element();
    }
    let mut row = div()
        .id(("stacked-pr-input", index))
        .debug_selector(move || format!("stacked-pr-input-{index}"))
        .flex()
        .items_center()
        .gap(px(5.))
        .font_family(fonts::mono())
        .text_size(px(11.))
        .text_color(rgb(t.fg_dim))
        .child(
            div()
                .min_w_0()
                .truncate()
                .child(SharedString::from(state.inputs[index].text().to_owned())),
        )
        .children(warning_icon)
        .child(icon(glyph::PENCIL_CIRCLE, 10., t.fg_faint));
    if editable {
        row = row
            .cursor_text()
            .on_click(cx.listener(move |view, _: &ClickEvent, _, cx| {
                view.activate_stacked_pr_input(index, cx)
            }));
    }
    row.into_any_element()
}
