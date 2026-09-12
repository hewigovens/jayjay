use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, rgb,
};

use super::RepoWindow;
use crate::app::config::{self, AppConfig};
use crate::app::theme::{Theme, ui_font_size};
use crate::ui::overlay::{overlay_actions, overlay_card, overlay_layer};
use crate::ui::primitives::{button, checkbox_row};

pub(crate) struct Confirmation {
    pub(crate) title: SharedString,
    pub(crate) message: SharedString,
    pub(crate) confirm_label: SharedString,
    pub(crate) action: ConfirmedAction,
    pub(crate) dont_ask_again: Option<DontAskAgain>,
}

/// The config flag a "Don't ask again" checkbox in the confirmation reads and toggles.
pub(crate) struct DontAskAgain {
    pub(crate) is_set: fn(&AppConfig) -> bool,
    pub(crate) toggle: fn(&mut AppConfig),
}

#[derive(Clone)]
pub(crate) enum ConfirmedAction {
    DeleteWorkspace { name: String, path: String },
    SquashChanges { revs: Vec<String> },
    AbandonChanges { revs: Vec<String> },
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
            ConfirmedAction::SquashChanges { revs } => {
                let task = self.vm.update(cx, |vm, cx| vm.squash_changes(revs, cx));
                task.detach();
            }
            ConfirmedAction::AbandonChanges { revs } => {
                let task = self.vm.update(cx, |vm, cx| vm.abandon_changes(revs, cx));
                task.detach();
            }
        }
    }
}

pub(super) fn confirmation_overlay(
    confirmation: &Confirmation,
    t: &Theme,
    cx: &mut Context<RepoWindow>,
) -> AnyElement {
    let mut card = overlay_card(t, 400.)
        .debug_selector(|| "confirmation".to_owned())
        .child(
            div()
                .text_size(ui_font_size(14.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgb(t.fg))
                .child(confirmation.title.clone()),
        )
        .child(
            div()
                .text_size(ui_font_size(12.))
                .text_color(rgb(t.fg_dim))
                .whitespace_normal()
                .child(confirmation.message.clone()),
        );
    if let Some(dont_ask_again) = &confirmation.dont_ask_again {
        let toggle = dont_ask_again.toggle;
        card = card.child(
            checkbox_row(
                "confirmation-dont-ask-again",
                "Don't ask again",
                (dont_ask_again.is_set)(&config::current(cx)),
                t,
            )
            .on_click(move |_, _, cx| config::update(cx, toggle)),
        );
    }
    overlay_layer()
        .child(
            card.child(overlay_actions(
                button("confirmation-cancel", "Cancel", t, false)
                    .debug_selector(|| "confirmation-cancel".to_owned())
                    .on_click(cx.listener(|view, _, _, cx| view.cancel_confirmation(cx))),
                button(
                    "confirmation-submit",
                    confirmation.confirm_label.clone(),
                    t,
                    true,
                )
                .debug_selector(|| "confirmation-submit".to_owned())
                .on_click(cx.listener(|view, _, _, cx| view.confirm(cx))),
            )),
        )
        .into_any_element()
}
