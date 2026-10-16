use gpui::{App, Context, ScrollStrategy, px};
use jayjay_core::diff::{DiffSpanStyle, build_diff_display_lines};

use super::RepoWindow;
use crate::app::fonts;
use crate::app::theme::theme;
use crate::diff::DiffViewMode;
use crate::diff::row_index_for_line;
use crate::diff::wrap::{
    sbs_line_to_row, visual_index_for_line, visual_index_for_sbs_row, wrap_cols_from_bounds,
};

impl RepoWindow {
    pub(super) fn scroll_diff_to_line(&self, line_ix: usize, cx: &App) {
        let vm = self.vm.read(cx);
        let advance = fonts::mono_advance(cx, px(theme(cx).code_font_size()));
        // Shared wrap helpers operate in u32; scroll_to_item takes usize.
        let line_ix_u32 = line_ix as u32;
        let item_ix = vm
            .shown
            .diff
            .as_ref()
            .map(|diff| {
                let display_lines = build_diff_display_lines(&diff.lines);
                let view_mode = vm.view_mode.effective_for_diff(Some(diff));
                let label_cols = self.diff.wrap_cache.borrow_mut().moved_label_cols(diff, cx);
                if view_mode == DiffViewMode::Unified {
                    let cols = wrap_cols_from_bounds(self.diff.unified_bounds.get(), advance);
                    let wrapped = self
                        .diff
                        .wrap_cache
                        .borrow_mut()
                        .unified(diff, cols, label_cols);
                    let w_ix = visual_index_for_line(&wrapped, line_ix_u32) as usize;
                    // Route through the row list `unified_body` renders, or a note above the target shifts the scroll.
                    self.diff_render_rows(cx)
                        .map(|rendered| row_index_for_line(&rendered.rows, w_ix))
                        .unwrap_or(w_ix)
                } else {
                    // SBS pairs Removed/Added and may wrap each side, so translate line_ix through pairing first, then through wrap.
                    let line_to_row = sbs_line_to_row(&display_lines);
                    let row_ix = line_to_row.get(line_ix).copied().unwrap_or(line_ix_u32);
                    let old_cols = wrap_cols_from_bounds(self.diff.sbs_old_bounds.get(), advance);
                    let new_cols = wrap_cols_from_bounds(self.diff.sbs_new_bounds.get(), advance);
                    let wrapped = self
                        .diff
                        .wrap_cache
                        .borrow_mut()
                        .side_by_side(diff, old_cols, new_cols, label_cols);
                    visual_index_for_sbs_row(&wrapped, row_ix) as usize
                }
            })
            .unwrap_or(line_ix);
        self.scrolls
            .diff
            .scroll_to_item(item_ix, ScrollStrategy::Center);
    }

    /// `line_no` is where the other copy starts, on the side opposite a `from` row.
    pub(crate) fn reveal_moved_counterpart(
        &mut self,
        from: DiffSpanStyle,
        line_no: u32,
        cx: &mut Context<Self>,
    ) {
        let Some(diff) = self.vm.read(cx).shown.diff.clone() else {
            return;
        };
        let target = build_diff_display_lines(&diff.lines)
            .iter()
            .position(|line| {
                if from == DiffSpanStyle::Removed {
                    line.style == DiffSpanStyle::Added && line.new_line_no == Some(line_no)
                } else {
                    line.style == DiffSpanStyle::Removed && line.old_line_no == Some(line_no)
                }
            });
        if let Some(line_ix) = target {
            self.scroll_diff_to_line(line_ix, cx);
            cx.notify();
        }
    }
}
