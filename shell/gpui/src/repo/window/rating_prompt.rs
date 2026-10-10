use gpui::{
    AnyElement, Context, FontWeight, InteractiveElement, IntoElement, KeyDownEvent, ParentElement,
    StatefulInteractiveElement, Styled, div, img, px, rgb,
};

use super::RepoWindow;
use crate::app::links::{self, REPOSITORY_URL, SPONSOR_URL};
use crate::app::rating_prompt::RatingPromptStore;
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::icons;
use crate::ui::overlay::{overlay_card, overlay_layer};
use crate::ui::primitives::button;

const MESSAGE: &str = "If JayJay is useful to you, a star on GitHub helps others find it, or sponsoring supports development.";

impl RepoWindow {
    pub(super) fn record_successful_action(&mut self, cx: &mut Context<Self>) {
        let can_show = !self.rating_prompt && !self.has_refresh_sensitive_interaction();
        if RatingPromptStore::record_action(cx, can_show) {
            self.rating_prompt = true;
            cx.notify();
        }
    }

    pub(super) fn handle_rating_prompt_key(
        &mut self,
        ev: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.rating_prompt {
            return false;
        }
        if ev.keystroke.key == "enter" && !ev.keystroke.modifiers.modified() {
            self.open_prompt_link(REPOSITORY_URL, cx);
        }
        true
    }

    pub(super) fn close_rating_prompt(&mut self, cx: &mut Context<Self>) {
        if std::mem::take(&mut self.rating_prompt) {
            cx.notify();
        }
    }

    fn open_prompt_link(&mut self, url: &str, cx: &mut Context<Self>) {
        links::open_url(cx, url);
        self.close_rating_prompt(cx);
    }
}

pub(super) fn rating_prompt_overlay(t: &Theme, cx: &mut Context<RepoWindow>) -> AnyElement {
    overlay_layer()
        .child(
            overlay_card(t, 340.)
                .debug_selector(|| "rating-prompt".to_owned())
                .items_center()
                .child(img(icons::HEART_CIRCLE_SVG).size(px(40.)))
                .child(
                    div()
                        .text_size(ui_font_size(16.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(t.fg))
                        .child("Enjoying JayJay?"),
                )
                .child(
                    div()
                        .text_size(ui_font_size(12.))
                        .text_color(rgb(t.fg_dim))
                        .text_center()
                        .whitespace_normal()
                        .child(MESSAGE),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(8.))
                        .pt(px(4.))
                        .child(
                            button("rating-prompt-star", "Star on GitHub", t, true)
                                .debug_selector(|| "rating-prompt-star".to_owned())
                                .on_click(cx.listener(|view, _, _, cx| {
                                    view.open_prompt_link(REPOSITORY_URL, cx)
                                })),
                        )
                        .child(
                            button("rating-prompt-sponsor", "Sponsor", t, false)
                                .debug_selector(|| "rating-prompt-sponsor".to_owned())
                                .on_click(cx.listener(|view, _, _, cx| {
                                    view.open_prompt_link(SPONSOR_URL, cx)
                                })),
                        ),
                )
                .child(
                    div()
                        .id("rating-prompt-dont-show")
                        .debug_selector(|| "rating-prompt-dont-show".to_owned())
                        .text_size(ui_font_size(11.))
                        .text_color(rgb(t.fg_faint))
                        .cursor_pointer()
                        .hover(|s| s.text_color(rgb(t.fg_dim)))
                        .child("Don't show again")
                        .on_click(cx.listener(|view, _, _, cx| {
                            RatingPromptStore::turn_off(cx);
                            view.close_rating_prompt(cx);
                        })),
                ),
        )
        .into_any_element()
}
