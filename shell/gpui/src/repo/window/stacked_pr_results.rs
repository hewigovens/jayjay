use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb, rgba,
};
use jayjay_core::StackLayerOutcome;

use super::RepoWindow;
use crate::app::fonts;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::primitives::button;

pub(super) fn results_body(
    result: &jayjay_core::StackedPrResult,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let mut list = div()
        .id("stacked-pr-results")
        .flex()
        .flex_col()
        .gap(px(6.))
        .max_h(px(300.))
        .overflow_y_scroll();
    for (index, layer) in result.layers.iter().enumerate() {
        let (label, color) = match layer.outcome {
            StackLayerOutcome::Created => ("Created", t.success_fg),
            StackLayerOutcome::Updated => ("Updated", t.toggle_active_fg),
            StackLayerOutcome::Failed => ("Failed", t.error_fg),
        };
        let mut title = div()
            .flex()
            .items_center()
            .gap(px(7.))
            .child(
                div()
                    .px(px(5.))
                    .py(px(2.))
                    .rounded_sm()
                    .bg(rgba((color << 8) | 0x22))
                    .text_size(ui_font_size(9.))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(color))
                    .child(label),
            )
            .child(if layer.title.is_empty() {
                layer.bookmark.clone()
            } else {
                layer.title.clone()
            });
        if layer.pr_number > 0 && !layer.pr_url.is_empty() {
            let url = layer.pr_url.clone();
            title = title.child(
                div()
                    .id(("stacked-pr-url", index))
                    .debug_selector(move || format!("stacked-pr-url-{index}"))
                    .cursor_pointer()
                    .text_color(rgb(t.toggle_active_fg))
                    .child(format!("#{}", layer.pr_number))
                    .on_click(move |_, _, cx| crate::app::links::open_url(cx, &url)),
            );
        }
        list = list.child(
            div()
                .id(("stacked-pr-result", index))
                .debug_selector(move || format!("stacked-pr-result-{index}"))
                .flex()
                .flex_col()
                .gap(px(4.))
                .px(px(10.))
                .py(px(8.))
                .rounded_md()
                .bg(rgb(t.row_alt_bg))
                .text_size(ui_font_size(12.))
                .child(title)
                .child(
                    div()
                        .font_family(fonts::mono())
                        .text_size(ui_font_size(10.))
                        .text_color(rgb(t.fg_dim))
                        .child(SharedString::from(layer.detail.clone())),
                ),
        );
    }
    div()
        .flex()
        .flex_col()
        .gap(px(10.))
        .child(list)
        .child(
            div()
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.fg_dim))
                .child(SharedString::from(result.message.clone())),
        )
        .child(
            div().flex().justify_end().child(
                button("stacked-pr-done", "Done", t, true)
                    .on_click(cx.listener(|view, _, _, cx| view.complete_stacked_pr(cx))),
            ),
        )
        .into_any_element()
}

pub(super) fn error_body(error: &str, t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(
            div()
                .px(px(9.))
                .py(px(8.))
                .rounded_md()
                .bg(rgba((t.error_fg << 8) | 0x18))
                .text_size(ui_font_size(11.))
                .text_color(rgb(t.error_fg))
                .child(SharedString::from(error.to_owned())),
        )
        .child(
            div()
                .flex()
                .justify_end()
                .gap(px(8.))
                .child(
                    button("stacked-pr-close", "Close", t, false)
                        .on_click(cx.listener(|view, _, _, cx| view.close_stacked_pr(cx))),
                )
                .child(
                    button("stacked-pr-retry", "Retry", t, true)
                        .on_click(cx.listener(|view, _, _, cx| view.retry_stacked_pr(cx))),
                ),
        )
        .into_any_element()
}

pub(super) fn centered_message(message: &'static str, t: &Theme) -> AnyElement {
    div()
        .h(px(80.))
        .flex()
        .items_center()
        .justify_center()
        .text_size(ui_font_size(12.))
        .text_color(rgb(t.fg_dim))
        .child(message)
        .into_any_element()
}
