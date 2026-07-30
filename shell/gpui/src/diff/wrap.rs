//! GPUI-shell glue around the shared wrap helpers in `jayjay_core::diff::wrap`.

use std::ops::Range;

use gpui::{Bounds, Pixels};

use jayjay_core::diff::{DEFAULT_WRAP_COLS, wrap_cols_for_width};
pub use jayjay_core::diff::{
    sbs_line_to_row, visual_index_for_line, visual_index_for_sbs_row, wrap_sbs_rows,
};

pub fn wrap_cols_from_bounds(bounds: Option<Bounds<Pixels>>, advance: Pixels) -> u32 {
    let Some(bounds) = bounds else {
        return DEFAULT_WRAP_COLS;
    };
    wrap_cols_for_width(f32::from(bounds.size.width), f32::from(advance))
}

pub fn selection_cols_in_fragment(
    cols: Range<usize>,
    fragment_start: usize,
    fragment_end: usize,
) -> Option<Range<usize>> {
    // `.then(|| ..)` is lazy — `.then_some` would underflow on out-of-range fragments.
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
        assert_eq!(selection_cols_in_fragment(5..10, 80, 150), None);
        assert_eq!(selection_cols_in_fragment(5..5, 80, 150), None);
        assert_eq!(selection_cols_in_fragment(200..210, 80, 150), None);
    }
}
