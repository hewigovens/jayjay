//! Per-window cache for wrapped diff lines, keyed on (`Arc<FileDiff>` identity, wrap cols); retaining the source Arc makes pointer identity stable and avoids re-wrapping (which deep-clones every `DiffLine`) on every mouse-move during a drag.

use std::ops::Deref;
use std::sync::Arc;

use jayjay_core::diff::{
    DiffLine, FileDiff, WrappedDiffLine, WrappedSbsRow, build_diff_display_lines,
    build_side_by_side_rows, wrap_diff_lines, wrap_sbs_rows,
};
use jayjay_review::ReviewNoteStatus;

use super::review_row_map::ReviewRowMap;
use super::rows::{DiffRenderRows, build_diff_render_rows, notes_fingerprint};

#[derive(Default)]
pub(crate) struct DiffWrapCache {
    unified: Option<UnifiedEntry>,
    sbs: Option<SbsEntry>,
    rows: Option<RowsEntry>,
    review: Option<ReviewEntry>,
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
}

struct SbsEntry {
    diff: Arc<FileDiff>,
    old_cols: u32,
    new_cols: u32,
    rows: Arc<CachedSbsRows>,
}

pub(crate) struct CachedSbsRows {
    rows: Vec<WrappedSbsRow>,
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
}

impl DiffWrapCache {
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

    pub(crate) fn unified(&mut self, fd: &Arc<FileDiff>, cols: u32) -> Arc<Vec<WrappedDiffLine>> {
        self.unified_entry(fd, cols).lines.clone()
    }

    fn unified_entry(&mut self, fd: &Arc<FileDiff>, cols: u32) -> &UnifiedEntry {
        let hit = self
            .unified
            .as_ref()
            .is_some_and(|entry| Arc::ptr_eq(&entry.diff, fd) && entry.cols == cols);
        if !hit {
            let display_lines = Arc::new(build_diff_display_lines(&fd.lines));
            let lines = Arc::new(wrap_diff_lines(&display_lines, cols));
            self.unified = Some(UnifiedEntry {
                diff: fd.clone(),
                cols,
                lines,
                display_lines,
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
    ) -> Arc<DiffRenderRows> {
        let fingerprint = notes_fingerprint(notes);
        if let Some(entry) = &self.rows
            && Arc::ptr_eq(&entry.diff, fd)
            && entry.cols == cols
            && entry.notes_fingerprint == fingerprint
        {
            return entry.rows.clone();
        }
        let entry = self.unified_entry(fd, cols);
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
            rows: rendered.clone(),
        });
        rendered
    }

    pub(crate) fn side_by_side(
        &mut self,
        fd: &Arc<FileDiff>,
        old_cols: u32,
        new_cols: u32,
    ) -> Arc<CachedSbsRows> {
        if let Some(entry) = &self.sbs
            && Arc::ptr_eq(&entry.diff, fd)
            && entry.old_cols == old_cols
            && entry.new_cols == new_cols
        {
            return entry.rows.clone();
        }
        let rows = build_side_by_side_rows(&fd.lines);
        let rows = Arc::new(CachedSbsRows {
            rows: wrap_sbs_rows(&rows, old_cols, new_cols),
        });
        self.sbs = Some(SbsEntry {
            diff: fd.clone(),
            old_cols,
            new_cols,
            rows: rows.clone(),
        });
        rows
    }
}

#[cfg(test)]
mod tests;
