use gpui::{App, Context, ScrollStrategy, px};

use super::RepoWindow;
use crate::app::fonts;
use crate::app::theme::theme;
use crate::diff::DiffViewMode;
use jayjay_core::diff::build_diff_display_lines;
use jayjay_core::diff::side_by_side::build_side_by_side_rows;

use crate::diff::row_index_for_line;
use crate::diff::wrap::{
    sbs_line_to_row, visual_index_for_line, visual_index_for_sbs_row, wrap_cols_from_bounds,
    wrap_sbs_rows,
};
use crate::ui::input::LineInput;

impl RepoWindow {
    fn find_input(view: &mut Self) -> Option<&mut LineInput> {
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
        let Some(diff) = vm.current_diff.as_ref() else {
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
            let vm = self.vm.read(cx);
            let advance = fonts::mono_advance(cx, px(theme(cx).font_size));
            // Shared wrap helpers operate in u32; scroll_to_item takes usize.
            let line_ix_u32 = line_ix as u32;
            let item_ix = vm
                .current_diff
                .as_ref()
                .map(|diff| {
                    let display_lines = build_diff_display_lines(&diff.lines);
                    let view_mode = vm.view_mode.effective_for_diff(Some(diff));
                    if view_mode == DiffViewMode::Unified {
                        let cols = wrap_cols_from_bounds(self.diff.unified_bounds.get(), advance);
                        let wrapped = self.diff.wrap_cache.borrow_mut().unified(diff, cols);
                        let w_ix = visual_index_for_line(&wrapped, line_ix_u32) as usize;
                        // Route through the same interleaved row list `unified_body` renders, not a private `wrap_diff_lines` call, so a note above the match can't shift the scroll target off by its own row count.
                        self.diff_render_rows(cx)
                            .map(|rendered| row_index_for_line(&rendered.rows, w_ix))
                            .unwrap_or(w_ix)
                    } else {
                        // SBS pairs Removed/Added and may wrap each side, so translate line_ix through pairing first, then through wrap.
                        let line_to_row = sbs_line_to_row(&display_lines);
                        let row_ix = line_to_row.get(line_ix).copied().unwrap_or(line_ix_u32);
                        let old_cols =
                            wrap_cols_from_bounds(self.diff.sbs_old_bounds.get(), advance);
                        let new_cols =
                            wrap_cols_from_bounds(self.diff.sbs_new_bounds.get(), advance);
                        let rows = build_side_by_side_rows(&display_lines);
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
            self.recompute_find_matches(cx);
            self.jump_to_current_match(cx);
        }
        LineInput::show_for_owner(self, cx, Self::find_input);
        cx.notify();
    }

    pub fn find_match_count(&self) -> usize {
        self.find.matches.len()
    }

    pub fn find_query_text(&self) -> Option<&str> {
        self.find.query.as_ref().map(LineInput::text)
    }
}
