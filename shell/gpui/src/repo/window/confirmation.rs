use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb, rgba,
};

use super::RepoWindow;
use crate::app::theme::Theme;
use crate::ui::primitives::button;

pub(crate) struct Confirmation {
    pub(crate) title: SharedString,
    pub(crate) message: SharedString,
    pub(crate) confirm_label: SharedString,
    pub(crate) action: ConfirmedAction,
}

#[derive(Clone)]
pub(crate) enum ConfirmedAction {
    DeleteWorkspace { name: String, path: String },
}

impl RepoWindow {
    pub(super) fn request_confirmation(
        &mut self,
        confirmation: Confirmation,
        cx: &mut Context<Self>,
    ) {
        self.close_bookmark_picker(cx);
        self.close_repo_switcher(cx);
        self.confirmation = Some(confirmation);
        cx.notify();
    }

    pub(crate) fn cancel_confirmation(&mut self, cx: &mut Context<Self>) {
        if self.confirmation.take().is_some() {
            cx.notify();
        }
    }

    pub(crate) fn confirm(&mut self, cx: &mut Context<Self>) {
        let Some(confirmation) = self.confirmation.take() else {
            return;
        };
        cx.notify();
        match confirmation.action {
            ConfirmedAction::DeleteWorkspace { name, path } => {
                self.delete_workspace(name, path, cx)
            }
        }
    }
}

pub(super) fn confirmation_overlay(
    confirmation: &Confirmation,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
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
                .w(px(400.))
                .px(px(18.))
                .py(px(16.))
                .rounded_lg()
                .border_1()
                .border_color(rgb(t.border))
                .bg(rgb(t.header_bg))
                .debug_selector(|| "confirmation".to_owned())
                .child(
                    div()
                        .text_size(px(14.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(t.fg))
                        .child(confirmation.title.clone()),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(rgb(t.fg_dim))
                        .whitespace_normal()
                        .child(confirmation.message.clone()),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .justify_end()
                        .gap(px(8.))
                        .child(
                            button("confirmation-cancel", "Cancel", t, false)
                                .debug_selector(|| "confirmation-cancel".to_owned())
                                .on_click(
                                    cx.listener(|view, _, _, cx| view.cancel_confirmation(cx)),
                                ),
                        )
                        .child(
                            button(
                                "confirmation-submit",
                                confirmation.confirm_label.clone(),
                                t,
                                true,
                            )
                            .debug_selector(|| "confirmation-submit".to_owned())
                            .on_click(cx.listener(|view, _, _, cx| view.confirm(cx))),
                        ),
                ),
        )
        .into_any_element()
}
