//! GPUI-shell glue around the shared wrap helpers in `jj-diff`.
//!
//! The wrap algorithm itself lives in `jayjay_core::diff::wrap` so the SwiftUI
//! shell (via uniffi) and the GPUI shell stay in lock-step. Anything that needs
//! gpui-specific types lives here.

use std::ops::Range;

use gpui::{Bounds, Pixels};

pub use jayjay_core::diff::{
    DEFAULT_WRAP_COLS, WrappedSbsRow, sbs_line_to_row, visual_index_for_line,
    visual_index_for_sbs_row, wrap_cols_for_width, wrap_diff_lines, wrap_sbs_rows,
};

/// Pull the pixel width out of a `Bounds<Pixels>` slot and forward to the shared
/// `wrap_cols_for_width`. Returns the default column count when the slot is empty.
pub fn wrap_cols_from_bounds(bounds: Option<Bounds<Pixels>>, advance: Pixels) -> u32 {
    let Some(bounds) = bounds else {
        return DEFAULT_WRAP_COLS;
    };
    wrap_cols_for_width(f32::from(bounds.size.width), f32::from(advance))
}

/// Map a logical selection column range onto a wrapped visual fragment. UI-only
/// helper — uses `Range<usize>` because the GPUI selection module is in usize
/// and this function lives next to its callers.
pub fn selection_cols_in_fragment(
    cols: Range<usize>,
    fragment_start: usize,
    fragment_end: usize,
) -> Option<Range<usize>> {
    // `.then_some(v)` is eager — `v` is evaluated even when the predicate is false.
    // Use `.then(|| v)` so the subtractions don't run on out-of-range fragments
    // (e.g. a selection on an earlier wrap segment vs. a later continuation fragment).
    if cols.start == cols.end {
        return (cols.start >= fragment_start && cols.start <= fragment_end)
            .then(|| (cols.start - fragment_start)..(cols.start - fragment_start));
    }

    let start = cols.start.max(fragment_start);
    let end = cols.end.min(fragment_end);
    (start < end).then(|| (start - fragment_start)..(end - fragment_start))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_is_mapped_relative_to_visual_fragment() {
        assert_eq!(selection_cols_in_fragment(2..8, 4, 10), Some(0..4));
        assert_eq!(selection_cols_in_fragment(2..4, 4, 10), None);
        assert_eq!(selection_cols_in_fragment(6..6, 4, 10), Some(2..2));
    }

    #[test]
    fn selection_before_continuation_fragment_returns_none_without_overflow() {
        // Regression: selection ends before this fragment starts.
        // Previously `.then_some(v)` evaluated `(end - fragment_start)` eagerly
        // and panicked with subtraction overflow.
        assert_eq!(selection_cols_in_fragment(5..10, 80, 150), None);
        assert_eq!(selection_cols_in_fragment(5..5, 80, 150), None);
        assert_eq!(selection_cols_in_fragment(200..210, 80, 150), None);
    }
}
