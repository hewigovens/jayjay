#[uniffi::export]
pub fn build_side_by_side_rows(
    lines: Vec<jayjay_core::diff::DiffLine>,
) -> Vec<jayjay_core::diff::SideBySideRow> {
    jayjay_core::diff::build_side_by_side_rows(&lines)
}

/// Wrap columns for a pane of `width` pixels with monospace `advance`.
#[uniffi::export]
pub fn wrap_cols_for_width(width: f32, advance: f32) -> u32 {
    jayjay_core::diff::wrap_cols_for_width(width, advance)
}

#[uniffi::export]
pub fn wrap_diff_lines(
    lines: Vec<jayjay_core::diff::DiffLine>,
    cols: u32,
) -> Vec<jayjay_core::diff::WrappedDiffLine> {
    jayjay_core::diff::wrap_diff_lines(&lines, cols)
}

#[uniffi::export]
pub fn wrap_sbs_rows(
    rows: Vec<jayjay_core::diff::SideBySideRow>,
    old_cols: u32,
    new_cols: u32,
) -> Vec<jayjay_core::diff::WrappedSbsRow> {
    jayjay_core::diff::wrap_sbs_rows(&rows, old_cols, new_cols)
}

#[uniffi::export]
pub fn sbs_line_to_row(lines: Vec<jayjay_core::diff::DiffLine>) -> Vec<u32> {
    jayjay_core::diff::sbs_line_to_row(&lines)
}

// `visual_index_for_line` / `visual_index_for_sbs_row` are intentionally NOT
// exposed via uniffi. Each call would copy the full wrapped Vec across the
// FFI boundary for a single lookup. When a Swift caller needs find-jump, the
// cleaner shape is to host the wrapped Vec on the Rust side behind a stateful
// object — until then the Rust API stays available to in-process consumers
// (the GPUI shell).
