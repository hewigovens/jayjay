//! Interleaves review-note rows into the unified diff's shared row list.

mod build_rows;
mod diff_render_rows;
mod fingerprint;
mod row_index;

pub use build_rows::build_diff_render_rows;
pub use diff_render_rows::{DiffRenderRow, DiffRenderRows, NoteDotKind};
pub(crate) use fingerprint::notes_fingerprint;
pub use row_index::row_index_for_line;

#[cfg(test)]
mod tests;
