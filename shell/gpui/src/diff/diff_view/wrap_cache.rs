//! Per-window cache for wrapped diff lines, keyed on (diff identity = live `Arc<FileDiff>` address, wrap cols); avoids re-wrapping (which deep-clones every `DiffLine`) on every mouse-move during a drag.

use std::sync::Arc;

use jayjay_core::diff::{
    DiffLine, FileDiff, WrappedDiffLine, WrappedSbsRow, build_diff_display_lines,
    build_side_by_side_rows, wrap_diff_lines, wrap_sbs_rows,
};
use jayjay_review::ReviewNoteStatus;

use super::rows::{DiffRenderRows, build_diff_render_rows, notes_fingerprint};

fn diff_identity(fd: &FileDiff) -> usize {
    fd as *const FileDiff as usize
}

#[derive(Default)]
pub(crate) struct DiffWrapCache {
    unified: Option<UnifiedEntry>,
    sbs: Option<SbsEntry>,
    rows: Option<RowsEntry>,
}

struct UnifiedEntry {
    identity: usize,
    cols: u32,
    lines: Arc<Vec<WrappedDiffLine>>,
    // The display lines `lines` was wrapped from; kept alongside so `rows()` can reuse them on a `unified()` hit instead of rebuilding the whole diff.
    display_lines: Arc<Vec<DiffLine>>,
}

struct SbsEntry {
    identity: usize,
    old_cols: u32,
    new_cols: u32,
    rows: Arc<Vec<WrappedSbsRow>>,
}

/// Keyed on (diff identity, cols, notes fingerprint); the fingerprint must change on both in-process mutations and external disk reloads that flip a note's reconciled status, which a local generation counter would miss.
struct RowsEntry {
    identity: usize,
    cols: u32,
    notes_fingerprint: u64,
    rows: Arc<DiffRenderRows>,
}

impl DiffWrapCache {
    pub(crate) fn unified(&mut self, fd: &FileDiff, cols: u32) -> Arc<Vec<WrappedDiffLine>> {
        self.unified_entry(fd, cols).lines.clone()
    }

    fn unified_entry(&mut self, fd: &FileDiff, cols: u32) -> &UnifiedEntry {
        let identity = diff_identity(fd);
        let hit = self
            .unified
            .as_ref()
            .is_some_and(|entry| entry.identity == identity && entry.cols == cols);
        if !hit {
            let display_lines = Arc::new(build_diff_display_lines(&fd.lines));
            let lines = Arc::new(wrap_diff_lines(&display_lines, cols));
            self.unified = Some(UnifiedEntry {
                identity,
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
        fd: &FileDiff,
        cols: u32,
        notes: &[ReviewNoteStatus],
    ) -> Arc<DiffRenderRows> {
        let identity = diff_identity(fd);
        let fingerprint = notes_fingerprint(notes);
        if let Some(entry) = &self.rows
            && entry.identity == identity
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
            identity,
            cols,
            notes_fingerprint: fingerprint,
            rows: rendered.clone(),
        });
        rendered
    }

    pub(crate) fn side_by_side(
        &mut self,
        fd: &FileDiff,
        old_cols: u32,
        new_cols: u32,
    ) -> Arc<Vec<WrappedSbsRow>> {
        let identity = diff_identity(fd);
        if let Some(entry) = &self.sbs
            && entry.identity == identity
            && entry.old_cols == old_cols
            && entry.new_cols == new_cols
        {
            return entry.rows.clone();
        }
        let rows = build_side_by_side_rows(&fd.lines);
        let rows = Arc::new(wrap_sbs_rows(&rows, old_cols, new_cols));
        self.sbs = Some(SbsEntry {
            identity,
            old_cols,
            new_cols,
            rows: rows.clone(),
        });
        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jayjay_core::diff::syntax::SyntaxToken;
    use jayjay_core::diff::{ConflictLineKind, DiffLine, DiffSpan, DiffSpanStyle};

    fn line(text: &str) -> DiffLine {
        DiffLine {
            old_line_no: Some(1),
            new_line_no: Some(1),
            style: DiffSpanStyle::Context,
            spans: vec![DiffSpan {
                text: text.to_owned(),
                style: DiffSpanStyle::Context,
                token: SyntaxToken::Plain,
            }],
            conflict_kind: ConflictLineKind::None,
            no_eof_newline: false,
        }
    }

    fn file_diff() -> FileDiff {
        FileDiff {
            path: "a.txt".to_owned(),
            language: "Text".to_owned(),
            lines: vec![line("hello"), line("world")],
            whitespace_only_hidden: false,
        }
    }

    #[test]
    fn unified_reuses_same_allocation_on_hit() {
        let fd = file_diff();
        let mut cache = DiffWrapCache::default();
        let first = cache.unified(&fd, 80);
        let second = cache.unified(&fd, 80);
        assert!(
            Arc::ptr_eq(&first, &second),
            "same key should reuse the Arc"
        );
    }

    #[test]
    fn unified_rewraps_when_cols_change() {
        let fd = file_diff();
        let mut cache = DiffWrapCache::default();
        let first = cache.unified(&fd, 80);
        let second = cache.unified(&fd, 40);
        assert!(!Arc::ptr_eq(&first, &second), "new cols should rewrap");
    }

    #[test]
    fn unified_rewraps_when_diff_identity_changes() {
        let mut cache = DiffWrapCache::default();
        let fd_a = file_diff();
        let first = cache.unified(&fd_a, 80);
        let fd_b = file_diff();
        let second = cache.unified(&fd_b, 80);
        assert!(
            !Arc::ptr_eq(&first, &second),
            "different diff should rewrap"
        );
    }

    #[test]
    fn side_by_side_reuses_same_allocation_on_hit() {
        let fd = file_diff();
        let mut cache = DiffWrapCache::default();
        let first = cache.side_by_side(&fd, 80, 80);
        let second = cache.side_by_side(&fd, 80, 80);
        assert!(
            Arc::ptr_eq(&first, &second),
            "same key should reuse the Arc"
        );
    }

    #[test]
    fn side_by_side_rewraps_when_a_side_changes() {
        let fd = file_diff();
        let mut cache = DiffWrapCache::default();
        let first = cache.side_by_side(&fd, 80, 80);
        let second = cache.side_by_side(&fd, 80, 40);
        assert!(!Arc::ptr_eq(&first, &second), "new cols should rewrap");
    }

    fn note(id: &str, resolved: bool) -> ReviewNoteStatus {
        use jayjay_review::{NoteEntry, NoteSide};
        ReviewNoteStatus {
            note: NoteEntry {
                id: id.to_owned(),
                change_id: "c1".to_owned(),
                path: "a.txt".to_owned(),
                identity: "id-1".to_owned(),
                side: NoteSide::New,
                line: 1,
                anchor_excerpt: String::new(),
                anchor_context: Vec::new(),
                ignore_whitespace: false,
                body: "note".to_owned(),
                created_at_ms: 0,
                updated_at_ms: 0,
                resolved,
                resolved_at_ms: None,
            },
            status: if resolved {
                jayjay_review::NoteStatus::Resolved
            } else {
                jayjay_review::NoteStatus::Current
            },
            group_index: Some(0),
        }
    }

    #[test]
    fn rows_reuses_same_allocation_when_notes_are_unchanged() {
        let fd = file_diff();
        let mut cache = DiffWrapCache::default();
        let notes = vec![note("n1", false)];
        let first = cache.rows(&fd, 80, &notes);
        let second = cache.rows(&fd, 80, &notes);
        assert!(
            Arc::ptr_eq(&first, &second),
            "identical notes should reuse the Arc"
        );
    }

    #[test]
    fn rows_rebuilds_when_a_note_changes_status() {
        let fd = file_diff();
        let mut cache = DiffWrapCache::default();
        let unresolved = vec![note("n1", false)];
        let resolved = vec![note("n1", true)];
        let first = cache.rows(&fd, 80, &unresolved);
        let second = cache.rows(&fd, 80, &resolved);
        assert!(
            !Arc::ptr_eq(&first, &second),
            "a status flip (e.g. an external resolve-note write) must rebuild the row list"
        );
    }

    #[test]
    fn rows_rebuilds_when_cols_change() {
        let fd = file_diff();
        let mut cache = DiffWrapCache::default();
        let notes = vec![note("n1", false)];
        let first = cache.rows(&fd, 80, &notes);
        let second = cache.rows(&fd, 40, &notes);
        assert!(!Arc::ptr_eq(&first, &second), "new cols should rebuild");
    }
}
