use gpui::{Context, Window};

use super::super::RepoWindow;
use crate::ui::input::LineInput;

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
        let revset = self.begin_ancestor_filter(&change_id, cx);
        self.select_change(index, cx);
        self.vm.update(cx, |vm, cx| vm.apply_revset(&revset, cx));
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
        let revset = self.begin_ancestor_filter(&head_change_id, cx);
        self.vm.update(cx, |vm, cx| {
            vm.apply_revset_selecting(&revset, select_commit_id, cx);
        });
        cx.notify();
    }

    fn begin_ancestor_filter(&mut self, change_id: &str, cx: &mut Context<Self>) -> String {
        self.show_sidebar(cx);
        if self.previous_ancestor_filter.is_none() {
            self.previous_ancestor_filter = Some(self.vm.read(cx).revset.to_string());
        }
        let revset = jayjay_core::ancestors_revset(change_id);
        if let Some(input) = self.revset_filter.as_mut() {
            input.set_text(revset.clone());
        } else {
            self.revset_filter = Some(LineInput::new(revset.clone()));
        }
        revset
    }

    fn revset_input(view: &mut Self) -> Option<&mut LineInput> {
        view.revset_filter.as_mut()
    }

    pub(crate) fn revset_filter_visible(&self) -> bool {
        self.revset_filter.is_some()
    }

    pub(crate) fn toggle_revset_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.revset_filter.is_some() && !self.layout.sidebar_hidden {
            self.close_revset_filter(cx);
            self.focus_handle.focus(window, cx);
            return;
        }
        self.show_sidebar(cx);
        if self.revset_filter.is_none() {
            let revset = self.vm.read(cx).revset.to_string();
            self.revset_filter = Some(LineInput::new(revset));
        }
        self.revset_filter_focus.focus(window, cx);
        LineInput::show_for_owner(self, cx, Self::revset_input);
        cx.on_next_frame(window, |view, _window, cx| {
            if let Some(input) = view.revset_filter.as_ref() {
                input.reveal_cursor_edge();
            }
            cx.notify();
        });
        cx.notify();
    }

    pub(in super::super) fn close_revset_filter(&mut self, cx: &mut Context<Self>) {
        LineInput::hide_for_owner(self, cx, Self::revset_input);
        self.revset_filter = None;
        cx.notify();
    }

    pub(super) fn apply_revset_filter(&mut self, cx: &mut Context<Self>) {
        let Some(input) = self.revset_filter.as_ref() else {
            return;
        };
        let revset = input.text().trim().to_owned();
        let revset = if revset.is_empty() {
            jayjay_core::build_default_revset(jayjay_core::DEFAULT_REVSET_DEPTH)
        } else {
            revset
        };
        self.apply_revset(&revset, cx);
    }

    pub(in super::super) fn apply_revset(&mut self, revset: &str, cx: &mut Context<Self>) {
        self.show_sidebar(cx);
        self.previous_ancestor_filter = None;
        if let Some(input) = self.revset_filter.as_mut() {
            input.set_text(revset);
        }
        self.vm.update(cx, |vm, cx| vm.apply_revset(revset, cx));
        LineInput::hide_for_owner(self, cx, Self::revset_input);
        cx.notify();
    }

    pub(super) fn reset_revset_filter(&mut self, cx: &mut Context<Self>) {
        if let Some(input) = self.revset_filter.as_mut() {
            input.clear();
        }
        self.apply_revset_filter(cx);
    }

    pub(in super::super) fn activate_revset_filter(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.revset_filter_focus.focus(window, cx);
        LineInput::show_for_owner(self, cx, Self::revset_input);
        cx.notify();
    }

    pub(super) fn handle_revset_filter_key(
        &mut self,
        ev: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(input) = self.revset_filter.as_mut() else {
            return false;
        };
        match ev.keystroke.key.as_str() {
            "escape" => {
                self.close_revset_filter(cx);
                self.focus_handle.focus(window, cx);
            }
            "enter" => {
                self.apply_revset_filter(cx);
                self.focus_handle.focus(window, cx);
            }
            _ => {
                let result = input.handle_key(ev, cx);
                if result.handled {
                    LineInput::show_for_owner(self, cx, Self::revset_input);
                    cx.notify();
                }
            }
        }
        true
    }
}
