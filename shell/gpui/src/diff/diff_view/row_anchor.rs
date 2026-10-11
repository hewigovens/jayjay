use std::sync::Arc;

use gpui::{UniformListScrollHandle, point, px};
use jayjay_core::diff::{FileDiff, WrappedDiffLine};

use super::rows::{DiffRenderRow, DiffRenderRows};
use super::wrap_cache::CachedSbsRows;

pub(crate) enum VisibleRows {
    Unified {
        diff: Arc<FileDiff>,
        lines: Arc<Vec<WrappedDiffLine>>,
        rendered: Arc<DiffRenderRows>,
    },
    SideBySide {
        diff: Arc<FileDiff>,
        rows: Arc<CachedSbsRows>,
    },
}

/// A source line and how far into its wrapped rows the view was, which survives reflow when the code font changes.
pub(crate) struct RowAnchor {
    diff: Arc<FileDiff>,
    source: u32,
    position: f32,
}

impl VisibleRows {
    fn diff(&self) -> &Arc<FileDiff> {
        match self {
            Self::Unified { diff, .. } | Self::SideBySide { diff, .. } => diff,
        }
    }

    /// Note rows belong to the line above them.
    fn sources(&self) -> Vec<u32> {
        match self {
            Self::Unified {
                lines, rendered, ..
            } => {
                let mut source = 0;
                rendered
                    .rows
                    .iter()
                    .map(|row| {
                        if let DiffRenderRow::Line(ix) = row {
                            source = lines[*ix].line_ix;
                        }
                        source
                    })
                    .collect()
            }
            Self::SideBySide { rows, .. } => rows.iter().map(|row| row.row_ix).collect(),
        }
    }

    /// `row` is fractional so a partly scrolled row keeps its offset.
    pub(crate) fn anchor(&self, row: f32) -> Option<RowAnchor> {
        let sources = self.sources();
        let source = *sources.get(row as usize)?;
        let (start, len) = source_rows(&sources, source)?;
        Some(RowAnchor {
            diff: self.diff().clone(),
            source,
            position: (row - start as f32) / len as f32,
        })
    }

    pub(crate) fn row(&self, anchor: &RowAnchor) -> Option<f32> {
        if !Arc::ptr_eq(self.diff(), &anchor.diff) {
            return None;
        }
        let (start, len) = source_rows(&self.sources(), anchor.source)?;
        Some(start as f32 + anchor.position * len as f32)
    }
}

fn source_rows(sources: &[u32], source: u32) -> Option<(usize, usize)> {
    let start = sources.iter().position(|&s| s == source)?;
    let len = sources[start..]
        .iter()
        .take_while(|&&s| s == source)
        .count();
    Some((start, len))
}

pub(super) fn scroll_to_row(scroll: &UniformListScrollHandle, row: f32, row_height: f32) {
    let base = scroll.0.borrow().base_handle.clone();
    base.set_offset(point(base.offset().x, px(-row * row_height)));
}
