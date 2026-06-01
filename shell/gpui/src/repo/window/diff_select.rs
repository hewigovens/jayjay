use gpui::Context;
use jayjay_core::diff::side_by_side::build_side_by_side_rows;

use super::RepoWindow;
use crate::diff::{DiffSelection, SbsSide, word_at};

impl RepoWindow {
    pub fn start_diff_selection(
        &mut self,
        line_ix: usize,
        col: usize,
        side: SbsSide,
        cx: &mut Context<Self>,
    ) {
        self.diff.selection = Some(DiffSelection::start(line_ix, col, side));
        cx.notify();
    }

    pub fn extend_diff_selection(
        &mut self,
        line_ix: usize,
        col: usize,
        side: SbsSide,
        cx: &mut Context<Self>,
    ) {
        let Some(sel) = self.diff.selection.as_mut() else {
            return;
        };
        if !sel.dragging || sel.side != side {
            return;
        }
        if sel.focus_line == line_ix && sel.focus_col == col {
            return;
        }
        sel.extend(line_ix, col);
        cx.notify();
    }

    pub fn finish_diff_selection(&mut self, cx: &mut Context<Self>) {
        let Some(sel) = self.diff.selection.as_mut() else {
            return;
        };
        sel.dragging = false;
        cx.notify();
    }

    pub fn select_word(
        &mut self,
        line_ix: usize,
        col: usize,
        side: SbsSide,
        cx: &mut Context<Self>,
    ) {
        let line_text = self.diff_line_text(line_ix, side, cx);
        let Some(text) = line_text else { return };
        let word = word_at(&text, col);
        let mut sel = DiffSelection::start(line_ix, word.start, side);
        sel.extend_to_word(line_ix, word);
        self.diff.selection = Some(sel);
        cx.notify();
    }

    pub fn copy_diff_selection(&mut self, cx: &mut Context<Self>) {
        let Some(sel) = self.diff.selection else {
            return;
        };
        let text = self.diff_selection_text(&sel, cx);
        if let Some(text) = text
            && !text.is_empty()
        {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
        }
    }

    fn diff_line_text(&self, line_ix: usize, side: SbsSide, cx: &Context<Self>) -> Option<String> {
        let fd = self.vm.read(cx).current_diff.as_ref()?;
        match side {
            SbsSide::Unified => {
                let line = fd.lines.get(line_ix)?;
                Some(line.spans.iter().map(|s| s.text.as_str()).collect())
            }
            SbsSide::Old | SbsSide::New => {
                let rows = build_side_by_side_rows(&fd.lines);
                let row = rows.get(line_ix)?;
                let spans = if matches!(side, SbsSide::Old) {
                    &row.old.spans
                } else {
                    &row.new.spans
                };
                Some(spans.iter().map(|s| s.text.as_str()).collect())
            }
        }
    }

    fn diff_selection_text(&self, sel: &DiffSelection, cx: &Context<Self>) -> Option<String> {
        let fd = self.vm.read(cx).current_diff.as_ref()?;
        let mut out: Vec<String> = Vec::new();
        match sel.side {
            SbsSide::Unified => {
                for ix in sel.line_range() {
                    let Some(line) = fd.lines.get(ix) else {
                        continue;
                    };
                    let text: String = line.spans.iter().map(|s| s.text.as_str()).collect();
                    let n = text.chars().count();
                    if let Some(cols) = sel.col_range_for(ix, n) {
                        out.push(slice_chars(&text, cols));
                    }
                }
            }
            SbsSide::Old | SbsSide::New => {
                let rows = build_side_by_side_rows(&fd.lines);
                for ix in sel.line_range() {
                    let Some(row) = rows.get(ix) else { continue };
                    let spans = if matches!(sel.side, SbsSide::Old) {
                        &row.old.spans
                    } else {
                        &row.new.spans
                    };
                    let text: String = spans.iter().map(|s| s.text.as_str()).collect();
                    let n = text.chars().count();
                    if let Some(cols) = sel.col_range_for(ix, n) {
                        out.push(slice_chars(&text, cols));
                    }
                }
            }
        }
        Some(out.join("\n"))
    }
}

fn slice_chars(text: &str, cols: std::ops::Range<usize>) -> String {
    text.chars()
        .skip(cols.start)
        .take(cols.end.saturating_sub(cols.start))
        .collect()
}
