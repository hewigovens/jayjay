use crate::types::MovedRow;

pub const DEFAULT_WRAP_COLS: u32 = 120;
pub const MIN_WRAP_COLS: u32 = 24;

/// Wrap columns for a pane of `width` pixels with monospace `advance`.
/// Returns `DEFAULT_WRAP_COLS` for non-positive inputs and clamps to `MIN_WRAP_COLS`.
pub fn wrap_cols_for_width(width: f32, advance: f32) -> u32 {
    if width <= 0. || advance <= 0. {
        return DEFAULT_WRAP_COLS;
    }
    ((width / advance).floor() as u32)
        .saturating_sub(1)
        .max(MIN_WRAP_COLS)
}

/// Narrows a moved block's first visual row so its jump label never covers code.
pub(super) fn cols_for(moved: Option<MovedRow>, cols: usize, label_cols: usize) -> usize {
    if moved.is_some_and(|moved| moved.starts_block) {
        cols.saturating_sub(label_cols).max(1)
    } else {
        cols
    }
}
