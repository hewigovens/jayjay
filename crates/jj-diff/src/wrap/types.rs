use crate::side_by_side::SideBySideRow;
use crate::types::DiffLine;

#[derive(Debug, Clone)]
pub struct WrappedDiffLine {
    pub line_ix: u32,
    pub line_len: u32,
    pub col_start: u32,
    pub col_end: u32,
    pub line: DiffLine,
}

/// Per-side wrap geometry for one visual row of a side-by-side diff.
#[derive(Debug, Clone, Default)]
pub struct WrappedSide {
    pub line_len: u32,
    pub col_start: u32,
    pub col_end: u32,
}

#[derive(Debug, Clone)]
pub struct WrappedSbsRow {
    pub row_ix: u32,
    pub old: WrappedSide,
    pub new: WrappedSide,
    pub row: SideBySideRow,
}
