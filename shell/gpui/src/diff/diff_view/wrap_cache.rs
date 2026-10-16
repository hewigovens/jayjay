//! Per-window cache for wrapped diff lines, keyed on (`Arc<FileDiff>` identity, wrap cols); retaining the source Arc makes pointer identity stable and avoids re-wrapping (which deep-clones every `DiffLine`) on every mouse-move during a drag.

use std::cell::OnceCell;
use std::ops::Deref;
use std::sync::Arc;

use gpui::{App, Pixels, WindowTextSystem, px};
use jayjay_core::FileSizeFormatter;
use jayjay_core::diff::{
    DiffLine, FileDiff, WrappedDiffLine, WrappedSbsRow, build_diff_display_lines,
    build_side_by_side_rows, sbs_line_to_row, wrap_diff_lines, wrap_sbs_rows,
};
use jayjay_review::ReviewNoteStatus;

use super::moved_label::{label_font_size, label_text, label_width};
use super::review_row_map::ReviewRowMap;
use super::row_anchor::{RowAnchor, VisibleRows};
use super::rows::{DiffRenderRows, build_diff_render_rows, notes_fingerprint};
use crate::app::{fonts, theme::theme};

#[derive(Default)]
pub(crate) struct DiffWrapCache {
    file_size_formatter: OnceCell<FileSizeFormatter>,
    unified: Option<UnifiedEntry>,
    sbs: Option<SbsEntry>,
    rows: Option<RowsEntry>,
    review: Option<ReviewEntry>,
    visible: Option<VisibleRows>,
    pinned: Option<RowAnchor>,
    moved_label: Option<MovedLabelMeasure>,
}

struct MovedLabelMeasure {
    diff: Arc<FileDiff>,
    family: String,
    size: Pixels,
    advance: Pixels,
    cols: u32,
}

struct ReviewEntry {
    diff: Arc<FileDiff>,
    rows: Arc<ReviewRowMap>,
}

struct UnifiedEntry {
    diff: Arc<FileDiff>,
    cols: u32,
    lines: Arc<Vec<WrappedDiffLine>>,
    // The display lines `lines` was wrapped from; kept alongside so `rows()` can reuse them on a `unified()` hit instead of rebuilding the whole diff.
    display_lines: Arc<Vec<DiffLine>>,
    label_cols: u32,
}

struct SbsEntry {
    diff: Arc<FileDiff>,
    old_cols: u32,
    new_cols: u32,
    rows: Arc<CachedSbsRows>,
    label_cols: u32,
}

pub(crate) struct CachedSbsRows {
    rows: Vec<WrappedSbsRow>,
    /// Per source row, the display line shown on the new side.
    new_display_lines: Vec<Option<usize>>,
}

impl CachedSbsRows {
    pub(crate) fn new_display_line(&self, ix: usize) -> Option<usize> {
        let row_ix = self.rows.get(ix)?.row_ix as usize;
        self.new_display_lines.get(row_ix).copied().flatten()
    }
}

impl Deref for CachedSbsRows {
    type Target = [WrappedSbsRow];

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

/// Keyed on (diff identity, cols, notes fingerprint); the fingerprint must change on both in-process mutations and external disk reloads that flip a note's reconciled status, which a local generation counter would miss.
struct RowsEntry {
    diff: Arc<FileDiff>,
    cols: u32,
    notes_fingerprint: u64,
    rows: Arc<DiffRenderRows>,
    label_cols: u32,
}

impl DiffWrapCache {
    pub(crate) fn moved_label_cols(&mut self, fd: &Arc<FileDiff>, cx: &App) -> u32 {
        let family = fonts::interface();
        let size = label_font_size(theme(cx));
        let advance = fonts::mono_advance(cx, px(theme(cx).code_font_size()));
        if let Some(entry) = &self.moved_label
            && Arc::ptr_eq(&entry.diff, fd)
            && entry.family == family
            && entry.size == size
            && entry.advance == advance
        {
            return entry.cols;
        }
        let text_system = WindowTextSystem::new(cx.text_system().clone());
        let width = fd
            .lines
            .iter()
            .filter_map(|line| {
                let moved = line.moved.filter(|moved| moved.starts_block)?;
                Some(label_width(
                    &label_text(moved, line.style),
                    size,
                    &text_system,
                ))
            })
            .max()
            .unwrap_or(px(0.));
        let cols = (f32::from(width) / f32::from(advance)).ceil() as u32;
        self.moved_label = Some(MovedLabelMeasure {
            diff: fd.clone(),
            family,
            size,
            advance,
            cols,
        });
        cols
    }

    fn file_size_formatter(&self) -> &FileSizeFormatter {
        self.file_size_formatter
            .get_or_init(|| FileSizeFormatter::new(&crate::platform::numeric_locale()))
    }

    /// Records the rows a diff body is showing; returns the row to scroll to when a pinned source line was waiting for this reflow.
    pub(crate) fn show(&mut self, rows: VisibleRows) -> Option<f32> {
        let row = self.pinned.take().and_then(|anchor| rows.row(&anchor));
        self.visible = Some(rows);
        row
    }

    pub(crate) fn hide(&mut self) {
        self.visible = None;
    }

    pub(crate) fn pin_row(&mut self, row: f32) {
        self.pinned = self.visible.as_ref().and_then(|rows| rows.anchor(row));
    }

    pub(crate) fn review_rows(&mut self, fd: &Arc<FileDiff>) -> Arc<ReviewRowMap> {
        if let Some(entry) = &self.review
            && Arc::ptr_eq(&entry.diff, fd)
        {
            return entry.rows.clone();
        }
        let rows = Arc::new(ReviewRowMap::new(fd));
        self.review = Some(ReviewEntry {
            diff: fd.clone(),
            rows: rows.clone(),
        });
        rows
    }

    pub(crate) fn unified(
        &mut self,
        fd: &Arc<FileDiff>,
        cols: u32,
        label_cols: u32,
    ) -> Arc<Vec<WrappedDiffLine>> {
        self.unified_entry(fd, cols, label_cols).lines.clone()
    }

    fn unified_entry(&mut self, fd: &Arc<FileDiff>, cols: u32, label_cols: u32) -> &UnifiedEntry {
        let hit = self.unified.as_ref().is_some_and(|entry| {
            Arc::ptr_eq(&entry.diff, fd) && entry.cols == cols && entry.label_cols == label_cols
        });
        if !hit {
            let mut display_lines = build_diff_display_lines(&fd.lines);
            for line in &mut display_lines {
                self.file_size_formatter()
                    .localize_binary_spans(&mut line.spans);
            }
            let display_lines = Arc::new(display_lines);
            let lines = Arc::new(wrap_diff_lines(&display_lines, cols, label_cols));
            self.unified = Some(UnifiedEntry {
                diff: fd.clone(),
                cols,
                lines,
                display_lines,
                label_cols,
            });
        }
        self.unified.as_ref().expect("just populated above")
    }

    /// Row list for `fd`'s unified rendering at `cols`, given notes already filtered to this hunk (see `RepoWindow::notes_for_selected_hunk`); reuses the same `unified` entry so `Line` row indices match the `Arc<Vec<WrappedDiffLine>>` callers separately fetch via `unified`.
    pub(crate) fn rows(
        &mut self,
        fd: &Arc<FileDiff>,
        cols: u32,
        notes: &[ReviewNoteStatus],
        label_cols: u32,
    ) -> Arc<DiffRenderRows> {
        let fingerprint = notes_fingerprint(notes);
        if let Some(entry) = &self.rows
            && Arc::ptr_eq(&entry.diff, fd)
            && entry.cols == cols
            && entry.label_cols == label_cols
            && entry.notes_fingerprint == fingerprint
        {
            return entry.rows.clone();
        }
        let entry = self.unified_entry(fd, cols, label_cols);
        let wrapped = entry.lines.clone();
        let display_lines = entry.display_lines.clone();
        let rendered = Arc::new(build_diff_render_rows(
            &wrapped,
            &display_lines,
            notes,
            cols,
        ));
        self.rows = Some(RowsEntry {
            diff: fd.clone(),
            cols,
            notes_fingerprint: fingerprint,
            label_cols,
            rows: rendered.clone(),
        });
        rendered
    }

    pub(crate) fn side_by_side(
        &mut self,
        fd: &Arc<FileDiff>,
        old_cols: u32,
        new_cols: u32,
        label_cols: u32,
    ) -> Arc<CachedSbsRows> {
        if let Some(entry) = &self.sbs
            && Arc::ptr_eq(&entry.diff, fd)
            && entry.old_cols == old_cols
            && entry.new_cols == new_cols
            && entry.label_cols == label_cols
        {
            return entry.rows.clone();
        }
        let mut rows = build_side_by_side_rows(&fd.lines);
        for row in &mut rows {
            self.file_size_formatter()
                .localize_binary_spans(&mut row.old.spans);
            self.file_size_formatter()
                .localize_binary_spans(&mut row.new.spans);
        }
        let mut new_display_lines = vec![None; rows.len()];
        let display_lines = build_diff_display_lines(&fd.lines);
        for (line_ix, row_ix) in sbs_line_to_row(&fd.lines).into_iter().enumerate() {
            if display_lines[line_ix].new_line_no.is_some() {
                new_display_lines[row_ix as usize] = Some(line_ix);
            }
        }
        let rows = Arc::new(CachedSbsRows {
            rows: wrap_sbs_rows(&rows, old_cols, new_cols, label_cols),
            new_display_lines,
        });
        self.sbs = Some(SbsEntry {
            diff: fd.clone(),
            old_cols,
            new_cols,
            label_cols,
            rows: rows.clone(),
        });
        rows
    }
}

#[cfg(test)]
mod tests;
