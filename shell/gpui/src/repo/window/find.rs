use gpui::{App, Context};

use super::RepoWindow;
use jayjay_core::diff::build_diff_display_lines;

use crate::ui::input::LineInput;

impl RepoWindow {
    pub(crate) fn find_input(view: &mut Self) -> Option<&mut LineInput> {
        view.find.query.as_mut()
    }

    pub fn open_find(&mut self, cx: &mut Context<Self>) {
        self.find.query = Some(LineInput::default());
        self.find.matches.clear();
        self.find.current = 0;
        LineInput::show_for_owner(self, cx, Self::find_input);
        cx.notify();
    }

    pub(crate) fn close_find(&mut self, cx: &mut Context<Self>) {
        LineInput::hide_for_owner(self, cx, Self::find_input);
        self.find.query = None;
        self.find.matches.clear();
        self.find.current = 0;
    }

    pub(super) fn recompute_find_matches(&mut self, cx: &App) {
        self.find.matches.clear();
        self.find.current = 0;
        let Some(query) = self.find.query.as_ref().map(LineInput::text) else {
            return;
        };
        if query.is_empty() {
            return;
        }
        let vm = self.vm.read(cx);
        let Some(diff) = vm.shown.diff.as_ref() else {
            return;
        };
        let q = query.to_lowercase();
        let display_lines = build_diff_display_lines(&diff.lines);
        for (ix, line) in display_lines.iter().enumerate() {
            let line_text: String = line.spans.iter().map(|s| s.text.as_str()).collect();
            if line_text.to_lowercase().contains(&q) {
                self.find.matches.push(ix);
            }
        }
    }

    fn jump_to_current_match(&self, cx: &App) {
        if let Some(&line_ix) = self.find.matches.get(self.find.current) {
            self.scroll_diff_to_line(line_ix, cx);
        }
    }

    pub(crate) fn find_advance(&mut self, prev: bool, cx: &mut Context<Self>) {
        if self.find.matches.is_empty() {
            cx.notify();
            return;
        }
        let len = self.find.matches.len();
        if prev {
            self.find.current = (self.find.current + len - 1) % len;
        } else {
            self.find.current = (self.find.current + 1) % len;
        }
        self.jump_to_current_match(cx);
        cx.notify();
    }

    pub(super) fn handle_find_key(
        &mut self,
        ev: &gpui::KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.find.query.is_none() {
            return false;
        }
        let key = ev.keystroke.key.as_str();
        let m = &ev.keystroke.modifiers;

        match key {
            "escape" => self.close_find(cx),
            "enter" | "f3" => self.find_advance(m.shift, cx),
            _ => self.handle_find_line_edit_key(ev, cx),
        }
        true
    }

    fn handle_find_line_edit_key(&mut self, ev: &gpui::KeyDownEvent, cx: &mut Context<Self>) {
        let Some(query) = self.find.query.as_mut() else {
            return;
        };
        let result = query.handle_key(ev, cx);
        if !result.handled {
            return;
        }
        if result.changed {
            self.find_query_edited(cx);
        }
        LineInput::show_for_owner(self, cx, Self::find_input);
        cx.notify();
    }

    pub(crate) fn find_query_edited(&mut self, cx: &mut Context<Self>) {
        self.recompute_find_matches(cx);
        self.jump_to_current_match(cx);
    }

    pub fn find_match_count(&self) -> usize {
        self.find.matches.len()
    }

    pub fn find_query_text(&self) -> Option<&str> {
        self.find.query.as_ref().map(LineInput::text)
    }
}
