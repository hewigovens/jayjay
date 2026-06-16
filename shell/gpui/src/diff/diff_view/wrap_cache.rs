//! Per-window cache for wrapped diff lines, keyed on `(diff identity, wrap cols)`.
//!
//! Wrapping deep-clones every `DiffLine`; without a cache, per-mouse-move notifies
//! during a drag would re-wrap the whole diff each event. Identity is the live
//! `Arc<FileDiff>` address, so a new diff (or new col count) rekeys.

use std::sync::Arc;

use jayjay_core::diff::{
    FileDiff, WrappedDiffLine, WrappedSbsRow, build_diff_display_lines, build_side_by_side_rows,
    wrap_diff_lines, wrap_sbs_rows,
};

/// Identity of a diff for cache keying: the `FileDiff` allocation address.
fn diff_identity(fd: &FileDiff) -> usize {
    fd as *const FileDiff as usize
}

#[derive(Default)]
pub(crate) struct DiffWrapCache {
    unified: Option<UnifiedEntry>,
    sbs: Option<SbsEntry>,
}

struct UnifiedEntry {
    identity: usize,
    cols: u32,
    lines: Arc<Vec<WrappedDiffLine>>,
}

struct SbsEntry {
    identity: usize,
    old_cols: u32,
    new_cols: u32,
    rows: Arc<Vec<WrappedSbsRow>>,
}

impl DiffWrapCache {
    /// Wrapped unified lines for `fd` at `cols`, reusing the cached value on a hit.
    pub(crate) fn unified(&mut self, fd: &FileDiff, cols: u32) -> Arc<Vec<WrappedDiffLine>> {
        let identity = diff_identity(fd);
        if let Some(entry) = &self.unified
            && entry.identity == identity
            && entry.cols == cols
        {
            return entry.lines.clone();
        }
        let display_lines = build_diff_display_lines(&fd.lines);
        let lines = Arc::new(wrap_diff_lines(&display_lines, cols));
        self.unified = Some(UnifiedEntry {
            identity,
            cols,
            lines: lines.clone(),
        });
        lines
    }

    /// Wrapped side-by-side rows for `fd` at the given per-side cols.
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
}
