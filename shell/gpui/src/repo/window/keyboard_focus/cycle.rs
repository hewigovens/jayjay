use gpui::{App, Context, Entity, Focusable, KeyDownEvent, Window};

use super::stop::FocusStop;
use super::visible::VisibleStops;
use crate::repo::window::{ActivePane, RepoWindow};
use crate::ui::text_area::TextArea;

impl RepoWindow {
    pub fn focused_control(&self) -> Option<FocusStop> {
        self.focused_control
    }

    pub(in crate::repo::window) fn sync_keyboard_focus(
        &mut self,
        window: &Window,
        cx: &App,
    ) -> bool {
        // Mouse focus and field submission can bypass the Tab cycle.
        let focused = self.focused_text_input(window, cx).or_else(|| {
            self.focused_control.filter(|stop| {
                !stop.is_text_input()
                    && self.focus_handle.is_focused(window)
                    && self.find.query.is_none()
            })
        });
        let changed = self.focused_control != focused;
        self.focused_control = focused;
        changed
    }

    /// Space and Return act only while a non-text control holds focus, so unfocused keys keep their pane meaning.
    pub(in crate::repo::window) fn handle_keyboard_focus_key(
        &mut self,
        ev: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.sync_keyboard_focus(window, cx) {
            cx.notify();
        }
        if self.keyboard_focus_suspended(cx) {
            return false;
        }
        let modifiers = &ev.keystroke.modifiers;
        if modifiers.platform || modifiers.alt || modifiers.control {
            return false;
        }
        if ev.keystroke.key == "tab" {
            self.move_keyboard_focus(modifiers.shift, window, cx);
            return true;
        }
        let Some(control) = self.focused_control else {
            return false;
        };
        if control.is_text_input() || !matches!(ev.keystroke.key.as_str(), "space" | "enter") {
            return false;
        }
        self.activate_focus_stop(control, window, cx);
        true
    }

    pub(in crate::repo::window) fn release_focused_control(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.sync_keyboard_focus(window, cx);
        if self.keyboard_focus_suspended(cx)
            || self.focused_control.is_some_and(FocusStop::is_text_input)
        {
            return false;
        }
        if self.focused_control.take().is_none() {
            return false;
        }
        cx.notify();
        true
    }

    /// Overlays, editors and modals own the keys while up; the panes are not rendered before a repo opens.
    fn keyboard_focus_suspended(&self, cx: &App) -> bool {
        let vm = self.vm.read(cx);
        self.has_refresh_sensitive_interaction()
            || self.find.query.is_some()
            || self.onboarding.is_some()
            || vm.repo.is_none()
            || vm.error.is_some()
    }

    fn move_keyboard_focus(&mut self, backward: bool, window: &mut Window, cx: &mut Context<Self>) {
        let current = self
            .focused_control
            .or_else(|| {
                self.file_filter_focus
                    .is_focused(window)
                    .then_some(FocusStop::FilterToggle)
            })
            .unwrap_or(match self.active_pane {
                ActivePane::Sidebar => FocusStop::Dag,
                ActivePane::FileColumn => FocusStop::FileList,
            });
        let next = self.visible_stops(cx).next(current, backward);
        // Drop any real text focus first; the next stop takes it back only when it is an input.
        self.focus_handle.focus(window, cx);
        let is_pane = matches!(next, FocusStop::Dag | FocusStop::FileList);
        self.focused_control = (!is_pane).then_some(next);
        if is_pane || next.is_text_input() {
            self.activate_focus_stop(next, window, cx);
        }
        cx.notify();
    }

    /// An input the user clicked into owns the cycle position, as if Tab had arrived there.
    pub(in crate::repo::window) fn focused_text_input(
        &self,
        window: &Window,
        cx: &App,
    ) -> Option<FocusStop> {
        let focused = |input: &Entity<TextArea>| input.read(cx).focus_handle(cx).is_focused(window);
        if focused(&self.commit_message.summary) {
            Some(FocusStop::CommitSummary)
        } else if focused(&self.commit_message.body) {
            Some(FocusStop::CommitDescription)
        } else if self.revset_filter_focus.is_focused(window) {
            Some(FocusStop::RevsetInput)
        } else {
            None
        }
    }

    fn activate_focus_stop(
        &mut self,
        stop: FocusStop,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match stop {
            FocusStop::Dag => self.active_pane = ActivePane::Sidebar,
            FocusStop::FileList => self.focus_file_list(cx),
            FocusStop::TreeToggle => {
                crate::app::config::update(cx, |config| config.diff.tree_file_list ^= true);
            }
            FocusStop::FilterToggle => self.toggle_file_filter(window, cx),
            FocusStop::ExpandDescription => {
                self.description.expanded ^= true;
                cx.notify();
            }
            FocusStop::DiffLayout => self.toggle_view_mode(cx),
            FocusStop::EditDescription => self.edit_selected_description(cx),
            FocusStop::EditDiff => self.enter_diff_edit(cx),
            FocusStop::RevsetFilter => self.toggle_revset_filter(window, cx),
            FocusStop::RevsetInput => self.activate_revset_filter(window, cx),
            FocusStop::Refresh => {
                let vm = self.vm.clone();
                vm.update(cx, |vm, cx| vm.refresh(false, cx));
            }
            FocusStop::Pull => self.git_fetch_origin(cx),
            FocusStop::Push => self.git_push_default(cx),
            FocusStop::Editor => self.open_repo_in_editor(cx),
            FocusStop::Terminal => self.open_repo_in_terminal(cx),
            FocusStop::Settings => crate::windows::settings::SettingsView::open(cx),
            FocusStop::CommitSummary => {
                let input = self.commit_message.summary.read(cx).focus_handle(cx);
                window.focus(&input, cx);
            }
            FocusStop::CommitDescription => {
                let input = self.commit_message.body.read(cx).focus_handle(cx);
                window.focus(&input, cx);
            }
        }
    }

    pub(in crate::repo::window) fn focus_file_list(&mut self, cx: &mut Context<Self>) {
        self.focused_control = None;
        self.active_pane = ActivePane::FileColumn;
        self.select_first_visible_file_if_needed(cx);
        cx.notify();
    }

    fn visible_stops(&self, cx: &App) -> VisibleStops {
        let vm = self.vm.read(cx);
        if vm.selection_without_diff_count().is_some() {
            return VisibleStops {
                revset_input: self.revset_filter.is_some(),
                ..VisibleStops::default()
            };
        }
        let detail_change = (vm.compare.is_none() && !vm.has_multiple_change_selection())
            .then(|| vm.selected_change())
            .flatten();
        let has_description =
            detail_change.is_some_and(|change| !change.description.trim().is_empty());
        VisibleStops {
            expand_description: has_description
                && (self.description.overflows || self.description.expanded),
            diff_layout: vm.selected_hunk().is_some(),
            edit_description: detail_change
                .is_some_and(|change| !change.is_immutable && !change.is_working_copy),
            edit_diff: detail_change.is_some_and(|change| {
                !change.has_conflict && !change.is_empty && !change.is_immutable
            }),
            revset_input: self.revset_filter.is_some(),
            commit_box: detail_change.is_some_and(|change| change.is_working_copy),
        }
    }
}
