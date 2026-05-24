use jayjay_core::diff::{self, DiffLine, SideBySideRow, WrappedDiffLine, WrappedSbsRow};

#[uniffi::export]
pub fn build_side_by_side_rows(lines: Vec<DiffLine>) -> Vec<SideBySideRow> {
    diff::build_side_by_side_rows(&lines)
}

/// Wrap columns for a pane of `width` pixels with monospace `advance`.
#[uniffi::export]
pub fn wrap_cols_for_width(width: f32, advance: f32) -> u32 {
    diff::wrap_cols_for_width(width, advance)
}

#[uniffi::export]
pub fn wrap_diff_lines(lines: Vec<DiffLine>, cols: u32) -> Vec<WrappedDiffLine> {
    diff::wrap_diff_lines(&lines, cols)
}

#[uniffi::export]
pub fn wrap_sbs_rows(
    rows: Vec<SideBySideRow>,
    old_cols: u32,
    new_cols: u32,
) -> Vec<WrappedSbsRow> {
    diff::wrap_sbs_rows(&rows, old_cols, new_cols)
}

#[uniffi::export]
pub fn sbs_line_to_row(lines: Vec<DiffLine>) -> Vec<u32> {
    diff::sbs_line_to_row(&lines)
}

// `visual_index_for_*` stay Rust-only — exporting would copy the full wrapped Vec across FFI per lookup.
