use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px, rgb, rgba,
};
use jayjay_core::ShortId;

use super::RepoWindow;
use super::dag_drag::DagRebaseRequest;
use crate::app::theme::Theme;
use crate::app::{config, fonts};
use crate::ui::icons::glyph;
use crate::ui::primitives::{button, checkbox_row, icon_label};

pub(super) fn rebase_confirmation_overlay(
    request: &DagRebaseRequest,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let toggle = checkbox_row(
        "rebase-confirm-toggle",
        "Confirm before drag-to-rebase",
        config::current(cx).features.confirm_drag_rebase,
        t,
    )
    .on_click(|_, _, cx| {
        config::update(cx, |config| config.features.confirm_drag_rebase ^= true);
    });

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
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(12.))
                .w(px(380.))
                .px(px(18.))
                .py(px(16.))
                .rounded_lg()
                .border_1()
                .border_color(rgb(t.border))
                .bg(rgb(t.header_bg))
                .debug_selector(|| "rebase-confirmation".to_owned())
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .child(
                            icon_label(glyph::ARROW_UP, "Rebase Change?", 16., t.fg_dim)
                                .text_size(px(14.))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(rgb(t.fg)),
                        )
                        .child(div().flex_1())
                        .child(
                            div()
                                .font_family(fonts::mono())
                                .text_size(px(11.))
                                .text_color(rgb(t.fg_dim))
                                .child(format!(
                                    "{} -> {}",
                                    request.source_commit_id.prefix(12),
                                    request.dest_commit_id.prefix(12)
                                )),
                        ),
                )
                .child(summary_row(
                    "Change",
                    &request.source_label,
                    &request.source_change_id,
                    t,
                ))
                .child(icon_label(
                    glyph::ARROW_DOWN,
                    "Will become a child of",
                    11.,
                    t.fg_dim,
                ))
                .child(summary_row(
                    "New parent",
                    &request.dest_label,
                    &request.dest_change_id,
                    t,
                ))
                .child(toggle)
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(rgb(t.fg_dim))
                        .child("Any conflicts will appear inline after the rebase."),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .justify_end()
                        .gap(px(8.))
                        .child(
                            button("rebase-confirm-cancel", "Cancel", t, false)
                                .debug_selector(|| "rebase-confirm-cancel".to_owned())
                                .on_click(
                                    cx.listener(|view, _, _, cx| view.cancel_drag_rebase(cx)),
                                ),
                        )
                        .child(
                            button("rebase-confirm-submit", "Rebase", t, true)
                                .debug_selector(|| "rebase-confirm-submit".to_owned())
                                .on_click(
                                    cx.listener(|view, _, _, cx| view.confirm_drag_rebase(cx)),
                                ),
                        ),
                ),
        )
        .into_any_element()
}

fn summary_row(title: &str, value: &str, detail: &ShortId, t: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(2.))
        .child(
            div()
                .text_size(px(11.))
                .text_color(rgb(t.fg_dim))
                .child(title.to_owned()),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_size(px(12.))
                        .text_color(rgb(t.fg))
                        .child(value.to_owned()),
                )
                .child(
                    div()
                        .font_family(fonts::mono())
                        .text_size(px(11.))
                        .text_color(rgb(t.fg_dim))
                        .child(detail.prefix(12)),
                ),
        )
        .into_any_element()
}
