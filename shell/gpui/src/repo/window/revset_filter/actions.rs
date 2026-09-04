use gpui::{Context, Window};

use super::super::RepoWindow;
use crate::ui::input::LineInput;

impl RepoWindow {
    fn revset_input(view: &mut Self) -> Option<&mut LineInput> {
        view.revset_filter.as_mut()
    }

    pub(crate) fn revset_filter_visible(&self) -> bool {
        self.revset_filter.is_some()
    }

    pub(crate) fn toggle_revset_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.revset_filter.is_some() {
            self.close_revset_filter(cx);
            self.focus_handle.focus(window, cx);
            return;
        }
        let revset = self.vm.read(cx).revset.to_string();
        self.revset_filter = Some(LineInput::new(revset));
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
        let Some(input) = self.revset_filter.as_mut() else {
            return;
        };
        let submitted = input.text().trim().to_owned();
        let use_default = submitted.is_empty() || {
            let vm = self.vm.read(cx);
            vm.revset_is_default() && vm.revset.as_ref() == submitted
        };
        let displayed = if use_default {
            jayjay_core::build_default_revset(jayjay_core::DEFAULT_LOG_CONTEXT_DEPTH)
        } else {
            submitted
        };
        input.set_text(displayed.clone());
        self.vm.update(cx, |vm, cx| {
            vm.apply_revset(if use_default { "" } else { &displayed }, cx)
        });
        LineInput::hide_for_owner(self, cx, Self::revset_input);
        cx.notify();
    }

    pub(super) fn select_revset_preset(&mut self, revset: &str, cx: &mut Context<Self>) {
        if let Some(input) = self.revset_filter.as_mut() {
            input.set_text(revset);
        }
        self.apply_revset_filter(cx);
    }

    pub(super) fn reset_revset_filter(&mut self, cx: &mut Context<Self>) {
        if let Some(input) = self.revset_filter.as_mut() {
            input.clear();
        }
        self.apply_revset_filter(cx);
    }

    pub(super) fn activate_revset_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
