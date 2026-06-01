use gpui::{App, Context, ScrollStrategy, px};

use super::RepoWindow;
use crate::app::fonts;
use crate::diff::DiffViewMode;
use jayjay_core::diff::side_by_side::build_side_by_side_rows;

use crate::diff::wrap::{
    sbs_line_to_row, visual_index_for_line, visual_index_for_sbs_row, wrap_cols_from_bounds,
    wrap_diff_lines, wrap_sbs_rows,
};

impl RepoWindow {
    pub fn open_find(&mut self, cx: &mut Context<Self>) {
        self.find.query = Some(String::new());
        self.find.matches.clear();
        self.find.current = 0;
        self.show_find_caret(cx);
        cx.notify();
    }

    pub fn close_find(&mut self, cx: &mut Context<Self>) {
        self.find.query = None;
        self.find.matches.clear();
        self.find.current = 0;
        self.find.caret.hide(cx);
    }

    fn show_find_caret(&mut self, cx: &mut Context<Self>) {
        self.find.caret.show(cx, |view, generation, cx| {
            view.toggle_find_caret(generation, cx)
        });
    }

    fn toggle_find_caret(&mut self, generation: u64, cx: &mut Context<Self>) -> bool {
        if self.find.query.is_none() {
            return false;
        }
        self.find.caret.toggle_if_current(generation, cx)
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

    fn jump_to_current_match(&self, cx: &App) {
        if let Some(&line_ix) = self.find.matches.get(self.find.current) {
            let vm = self.vm.read(cx);
            let advance = fonts::mono_advance(cx, px(12.));
            // Shared wrap helpers operate in u32; scroll_to_item takes usize.
            let line_ix_u32 = line_ix as u32;
            let item_ix = vm
                .current_diff
                .as_ref()
                .map(|diff| {
                    if vm.view_mode == DiffViewMode::Unified {
                        let cols = wrap_cols_from_bounds(self.diff.unified_bounds.get(), advance);
                        visual_index_for_line(&wrap_diff_lines(&diff.lines, cols), line_ix_u32)
                            as usize
                    } else {
                        // SBS pairs Removed/Added and may wrap each side — translate
                        // line_ix through pairing first, then through wrap.
                        let line_to_row = sbs_line_to_row(&diff.lines);
                        let row_ix = line_to_row.get(line_ix).copied().unwrap_or(line_ix_u32);
                        let old_cols =
                            wrap_cols_from_bounds(self.diff.sbs_old_bounds.get(), advance);
                        let new_cols =
                            wrap_cols_from_bounds(self.diff.sbs_new_bounds.get(), advance);
                        let rows = build_side_by_side_rows(&diff.lines);
                        let wrapped = wrap_sbs_rows(&rows, old_cols, new_cols);
                        visual_index_for_sbs_row(&wrapped, row_ix) as usize
                    }
                })
                .unwrap_or(line_ix);
            self.scrolls
                .diff
                .scroll_to_item(item_ix, ScrollStrategy::Center);
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

        if m.platform && key == "v" {
            if let Some(item) = cx.read_from_clipboard()
                && let Some(text) = item.text()
                && let Some(q) = self.find.query.as_mut()
            {
                q.push_str(&text);
                self.recompute_find_matches(cx);
                self.jump_to_current_match(cx);
                self.show_find_caret(cx);
                cx.notify();
            }
            return true;
        }

        if m.platform || m.control || m.alt {
            return false;
        }

        match key {
            "escape" => self.close_find(cx),
            "enter" | "f3" => self.find_advance(m.shift, cx),
            "backspace" => {
                if let Some(q) = self.find.query.as_mut() {
                    q.pop();
                    self.recompute_find_matches(cx);
                    self.jump_to_current_match(cx);
                    self.show_find_caret(cx);
                    cx.notify();
                }
            }
            _ => {
                if let Some(c) = ev.keystroke.key_char.as_ref() {
                    let printable = !c.is_empty() && c.chars().all(|ch| !ch.is_control());
                    if printable && let Some(q) = self.find.query.as_mut() {
                        q.push_str(c);
                        self.recompute_find_matches(cx);
                        self.jump_to_current_match(cx);
                        self.show_find_caret(cx);
                        cx.notify();
                    }
                }
            }
        }
        true
    }
}
