//! Visual wrapping for unified and side-by-side diffs.
//!
//! Shells render the diff in monospace; long lines either scroll horizontally or
//! wrap to multiple visual rows. These helpers compute the wrapped layout in
//! pure Rust so both the GPUI and AppKit shells share the same row/column math
//! (and therefore keep the gutter and panes vertically aligned).
//!
//! Pure functions, no UI framework. `f32` widths cross the uniffi boundary into Swift.

mod chunks;
mod cols;
mod side_by_side;
mod types;
mod unified;

#[cfg(test)]
mod tests;

pub use cols::{DEFAULT_WRAP_COLS, wrap_cols_for_width};
pub use side_by_side::{sbs_line_to_row, visual_index_for_sbs_row, wrap_sbs_rows};
pub use types::{WrappedDiffLine, WrappedSbsRow, WrappedSide};
pub use unified::{visual_index_for_line, wrap_diff_lines};
