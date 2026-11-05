use super::{ChangeAction, RepoWindow};
use crate::app::actions::{
    AbandonChange, DescribeChange, EditChange, NewBookmark, NewChange, OpenBisect, RebaseOntoTrunk,
    ShowEvolution, SquashChange,
};
use crate::ui::context_menu::ContextAction;
use gpui::{Context, Div, InteractiveElement};
use jayjay_core::shortcuts::ShortcutCommand;
use std::sync::Arc;

impl RepoWindow {
    pub(super) fn change_shortcuts(root: Div, cx: &mut Context<Self>) -> Div {
        root.on_action(cx.listener(|view, _: &NewChange, _, cx| {
            view.run_shortcut(ShortcutCommand::RepoNewChange, cx)
        }))
        .on_action(cx.listener(|view, _: &EditChange, _, cx| {
            view.run_shortcut(ShortcutCommand::RepoEditChange, cx)
        }))
        .on_action(cx.listener(|view, _: &DescribeChange, _, cx| {
            view.run_shortcut(ShortcutCommand::RepoDescribeChange, cx)
        }))
        .on_action(cx.listener(|view, _: &SquashChange, _, cx| {
            view.run_shortcut(ShortcutCommand::RepoSquashChange, cx)
        }))
        .on_action(cx.listener(|view, _: &NewBookmark, _, cx| {
            view.run_shortcut(ShortcutCommand::RepoNewBookmark, cx)
        }))
        .on_action(cx.listener(|view, _: &ShowEvolution, _, cx| {
            view.run_shortcut(ShortcutCommand::RepoShowEvolution, cx)
        }))
        .on_action(cx.listener(|view, _: &AbandonChange, _, cx| {
            view.run_shortcut(ShortcutCommand::ChangesAbandon, cx)
        }))
        .on_action(cx.listener(|view, _: &OpenBisect, _, cx| view.open_bisect_sheet(None, cx)))
        .on_action(cx.listener(|view, _: &RebaseOntoTrunk, _, cx| {
            let rev = "@".to_owned();
            view.run_change_action(Arc::new(ChangeAction::RebaseOntoTrunk { rev }), cx)
        }))
    }

    pub(super) fn run_shortcut(&mut self, command: ShortcutCommand, cx: &mut Context<Self>) {
        let Some(change) = self.vm.read(cx).selected_change().cloned() else {
            return;
        };
        let item = self
            .build_change_menu(&change, cx)
            .into_iter()
            .find(|item| {
                if !item.enabled {
                    return false;
                }
                match (&item.action, command) {
                    (ContextAction::NewChangeOnTop(_), ShortcutCommand::RepoNewChange)
                    | (
                        ContextAction::EditDescription { .. },
                        ShortcutCommand::RepoDescribeChange,
                    )
                    | (ContextAction::CreateBookmark(_), ShortcutCommand::RepoNewBookmark)
                    | (ContextAction::OpenEvologFor(_), ShortcutCommand::RepoShowEvolution)
                    | (ContextAction::AbandonChange(_), ShortcutCommand::ChangesAbandon) => true,
                    (ContextAction::Change(action), command) => matches!(
                        (action.as_ref(), command),
                        (ChangeAction::Edit { .. }, ShortcutCommand::RepoEditChange)
                            | (
                                ChangeAction::Squash { into: None, .. }
                                    | ChangeAction::SquashMany { .. },
                                ShortcutCommand::RepoSquashChange
                            )
                            | (
                                ChangeAction::AbandonMany { .. },
                                ShortcutCommand::ChangesAbandon
                            )
                    ),
                    _ => false,
                }
            });
        if let Some(item) = item {
            self.dispatch_context_action(item.action, cx);
        }
    }
}
