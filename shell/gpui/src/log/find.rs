use gpui::{App, Context, ScrollStrategy};

use super::LogView;

impl LogView {
    pub fn open_find(&mut self, cx: &mut Context<Self>) {
        self.find.query = Some(String::new());
        self.find.matches.clear();
        self.find.current = 0;
        cx.notify();
    }

    pub fn close_find(&mut self, cx: &mut Context<Self>) {
        self.find.query = None;
        self.find.matches.clear();
        self.find.current = 0;
        cx.notify();
    }

    pub(super) fn recompute_find_matches(&mut self, cx: &App) {
        self.find.matches.clear();
        self.find.current = 0;
        let Some(query) = self.find.query.as_ref() else {
            return;
        };
        if query.is_empty() {
            return;
        }
        let vm = self.vm.read(cx);
        let Some(diff) = vm.current_diff.as_ref() else {
            return;
        };
        let q = query.to_lowercase();
        for (ix, line) in diff.lines.iter().enumerate() {
            let line_text: String = line.spans.iter().map(|s| s.text.as_str()).collect();
            if line_text.to_lowercase().contains(&q) {
                self.find.matches.push(ix);
            }
        }
    }

    fn jump_to_current_match(&self) {
        if let Some(&line_ix) = self.find.matches.get(self.find.current) {
            self.scrolls.diff
                .scroll_to_item(line_ix, ScrollStrategy::Center);
        }
    }

    pub fn find_advance(&mut self, prev: bool, cx: &mut Context<Self>) {
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
        self.jump_to_current_match();
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

        if m.platform && key == "v" {
            if let Some(item) = cx.read_from_clipboard()
                && let Some(text) = item.text()
                && let Some(q) = self.find.query.as_mut()
            {
                q.push_str(&text);
                self.recompute_find_matches(cx);
                cx.notify();
            }
            return true;
        }

        if m.platform || m.control || m.alt {
            return false;
        }

        match key {
            "escape" => self.close_find(cx),
            "enter" => self.find_advance(m.shift, cx),
            "backspace" => {
                if let Some(q) = self.find.query.as_mut() {
                    q.pop();
                    self.recompute_find_matches(cx);
                    cx.notify();
                }
            }
            _ => {
                if let Some(c) = ev.keystroke.key_char.as_ref() {
                    let printable = !c.is_empty() && c.chars().all(|ch| !ch.is_control());
                    if printable && let Some(q) = self.find.query.as_mut() {
                        q.push_str(c);
                        self.recompute_find_matches(cx);
                        cx.notify();
                    }
                }
            }
        }
        true
    }
}
