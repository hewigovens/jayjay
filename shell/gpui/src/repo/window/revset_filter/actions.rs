use gpui::{Context, KeyDownEvent, Window};
use jayjay_core::{RevsetSuggestion, RevsetSuggestionKind, typed_revset};

use super::super::{FocusStop, RepoWindow};
use super::popup::RevsetPopupState;
use crate::app::error_text;
use crate::ui::input::LineInput;
use crate::ui::navigation::{ListNav, ListNavKeys, list_nav_from_key};

impl RepoWindow {
    pub(in super::super) fn show_ancestors(&mut self, commit_id: &str, cx: &mut Context<Self>) {
        let Some((index, change_id)) = self
            .vm
            .read(cx)
            .graph
            .changes
            .iter()
            .enumerate()
            .find(|(_, change)| change.commit_id.id == commit_id)
            .map(|(index, change)| (index, change.change_id.id.clone()))
        else {
            return;
        };
        self.show_sidebar(cx);
        self.select_change(index, cx);
        self.vm
            .update(cx, |vm, cx| vm.show_ancestors(&change_id, None, cx));
        cx.notify();
    }

    /// A window still opening applies the reveal once its repository has loaded.
    pub(crate) fn reveal_ancestors(
        &mut self,
        head_change_id: String,
        select_commit_id: String,
        cx: &mut Context<Self>,
    ) {
        if self.vm.read(cx).repo.is_none() {
            self.pending_reveal = Some((head_change_id, select_commit_id));
            return;
        }
        self.show_sidebar(cx);
        self.vm.update(cx, |vm, cx| {
            vm.show_ancestors(&head_change_id, Some(select_commit_id), cx);
        });
        cx.notify();
    }

    pub(in super::super) fn apply_revset(&mut self, revset: &str, cx: &mut Context<Self>) {
        self.apply_revset_selecting(revset, None, cx);
    }

    pub(in super::super) fn apply_revset_selecting(
        &mut self,
        revset: &str,
        selecting: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.show_sidebar(cx);
        self.close_revset_editor(cx);
        self.revset_popup = None;
        self.vm.update(cx, |vm, cx| match selecting {
            Some(commit_id) => vm.apply_revset_selecting(revset, commit_id, cx),
            None => vm.apply_revset(revset, cx),
        });
        cx.notify();
    }

    pub(in super::super) fn return_to_previous_revset(&mut self, cx: &mut Context<Self>) {
        self.show_sidebar(cx);
        self.vm
            .update(cx, |vm, cx| vm.return_to_previous_revset(cx));
        cx.notify();
    }

    /// A preset's name or a bookmark picks that filter; anything else must parse before the graph reloads.
    fn apply_typed_revset(&mut self, text: &str, cx: &mut Context<Self>) -> Result<(), String> {
        let (revset, repo) = {
            let vm = self.vm.read(cx);
            (typed_revset(text, &vm.graph.bookmarks), vm.repo.clone())
        };
        if revset == text
            && let Some(repo) = repo
            && let Err(error) = repo.check_revset(text)
        {
            return Err(error_text(error).to_string());
        }
        self.apply_revset(&revset, cx);
        Ok(())
    }

    pub(crate) fn begin_revset_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.revset_popup = None;
        let revset = self.vm.read(cx).revset().to_owned();
        self.revset_editor = Some(LineInput::new(revset));
        self.revset_editor_had_focus = false;
        self.revset_editor_focus.focus(window, cx);
        LineInput::show_for_owner(self, cx, Self::revset_editor_input);
        cx.notify();
    }

    /// The palette has no window to focus with, so the next render opens the editor.
    pub(crate) fn request_revset_edit(&mut self, cx: &mut Context<Self>) {
        self.revset_edit_requested = true;
        cx.notify();
    }

    /// Opens a palette-requested editor, and cancels the edit once focus has moved elsewhere, including a title-bar click.
    pub(in super::super) fn sync_revset_editor(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if std::mem::take(&mut self.revset_edit_requested) {
            self.begin_revset_edit(window, cx);
        }
        if self.revset_editor.is_none() {
            // A closed editor's handle keeps focus until something else takes it, and would read as a clicked-in input.
            if self.revset_editor_focus.is_focused(window) {
                self.focus_handle.focus(window, cx);
            }
            return;
        }
        if self.revset_editor_focus.is_focused(window) {
            self.revset_editor_had_focus = true;
        } else if self.revset_editor_had_focus {
            self.close_revset_editor(cx);
        }
    }

    pub(in super::super) fn close_revset_editor(&mut self, cx: &mut Context<Self>) {
        if self.revset_editor.is_some() {
            LineInput::hide_for_owner(self, cx, Self::revset_editor_input);
            self.revset_editor = None;
            // A clicked-in edit must not leave a focus ring behind; Tab moved it on already.
            if self.focused_control == Some(FocusStop::RevsetFilter) {
                self.focused_control = None;
            }
            cx.notify();
        }
    }

    fn revset_editor_input(view: &mut Self) -> Option<&mut LineInput> {
        view.revset_editor.as_mut()
    }

    pub(super) fn handle_revset_editor_key(
        &mut self,
        ev: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(input) = self.revset_editor.as_mut() else {
            return false;
        };
        match ev.keystroke.key.as_str() {
            "escape" => {
                self.close_revset_editor(cx);
                self.focus_handle.focus(window, cx);
            }
            "enter" => {
                let text = input.text().trim().to_owned();
                self.close_revset_editor(cx);
                self.focus_handle.focus(window, cx);
                if !text.is_empty()
                    && text != self.vm.read(cx).revset()
                    && let Err(error) = self.apply_typed_revset(&text, cx)
                {
                    self.open_revset_popup(text, Some(error), window, cx);
                }
            }
            _ => {
                if !input.handle_key(ev, cx).handled {
                    return false;
                }
                LineInput::show_for_owner(self, cx, Self::revset_editor_input);
                cx.notify();
            }
        }
        true
    }

    pub(in super::super) fn toggle_revset_popup(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.revset_popup.is_some() {
            self.close_revset_popup(cx);
        } else {
            self.open_revset_popup(String::new(), None, window, cx);
        }
    }

    /// The popup's keys arrive through the window, so any text field that held focus must give it up.
    fn open_revset_popup(
        &mut self,
        query: String,
        error: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_handle.focus(window, cx);
        self.close_revset_editor(cx);
        self.close_bookmark_picker(cx);
        self.revset_popup = Some(RevsetPopupState::new(query, error));
        LineInput::show_for_owner(self, cx, Self::revset_popup_input);
        cx.notify();
    }

    pub(crate) fn close_revset_popup(&mut self, cx: &mut Context<Self>) {
        if self.revset_popup.is_some() {
            LineInput::hide_for_owner(self, cx, Self::revset_popup_input);
            self.revset_popup = None;
            cx.notify();
        }
    }

    fn revset_popup_input(view: &mut Self) -> Option<&mut LineInput> {
        view.revset_popup
            .as_mut()
            .map(|popup| &mut popup.query.input)
    }

    pub(in super::super) fn revset_suggestions(&self, cx: &gpui::App) -> Vec<RevsetSuggestion> {
        let Some(popup) = self.revset_popup.as_ref() else {
            return Vec::new();
        };
        let vm = self.vm.read(cx);
        vm.revset_filter
            .suggestions(popup.query.input.text(), &vm.graph.bookmarks)
    }

    /// The current revset loads into the field for editing; every other row applies.
    pub(super) fn activate_revset_suggestion(
        &mut self,
        suggestion: RevsetSuggestion,
        cx: &mut Context<Self>,
    ) {
        if suggestion.kind == RevsetSuggestionKind::Current {
            if let Some(popup) = self.revset_popup.as_mut() {
                popup.query.input.set_text(suggestion.revset);
                popup.query.selected = None;
                popup.error = None;
            }
            cx.notify();
        } else {
            self.apply_revset(&suggestion.revset, cx);
        }
    }

    pub(in super::super) fn handle_revset_popup_key(
        &mut self,
        ev: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        let suggestions = self.revset_suggestions(cx);
        let Some(popup) = self.revset_popup.as_mut() else {
            return false;
        };
        if let Some(direction) = list_nav_from_key(ev, ListNavKeys::COMMAND_PALETTE) {
            if !suggestions.is_empty() {
                let last = suggestions.len() - 1;
                popup.query.selected = Some(match (popup.query.selected, direction) {
                    (None, _) => 0,
                    (Some(index), ListNav::Previous) => index.saturating_sub(1),
                    (Some(index), ListNav::Next) => (index + 1).min(last),
                });
            }
            cx.notify();
            return true;
        }
        match ev.keystroke.key.as_str() {
            "escape" => self.close_revset_popup(cx),
            "enter" => {
                if let Some(suggestion) = popup
                    .query
                    .selected
                    .and_then(|index| suggestions.get(index).cloned())
                {
                    self.activate_revset_suggestion(suggestion, cx);
                    return true;
                }
                let text = popup.query.input.text().trim().to_owned();
                if text.is_empty() {
                    self.close_revset_popup(cx);
                } else if let Err(error) = self.apply_typed_revset(&text, cx)
                    && let Some(popup) = self.revset_popup.as_mut()
                {
                    popup.error = Some(error);
                    cx.notify();
                }
            }
            _ => {
                if popup.query.input.handle_key(ev, cx).handled {
                    popup.query.selected = None;
                    popup.error = None;
                    LineInput::show_for_owner(self, cx, Self::revset_popup_input);
                    cx.notify();
                }
            }
        }
        true
    }
}
