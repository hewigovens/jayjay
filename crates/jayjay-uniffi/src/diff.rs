use jayjay_core::diff::{
    self, ChangeGroup, ConflictLineKind, DiffLine, DiffSpan, SideBySideRow, WrappedDiffLine,
    WrappedSbsRow,
};

#[uniffi::export]
pub fn build_side_by_side_rows(lines: Vec<DiffLine>) -> Vec<SideBySideRow> {
    diff::build_side_by_side_rows(&lines)
}

#[uniffi::export]
pub fn wrap_cols_for_width(width: f32, advance: f32) -> u32 {
    diff::wrap_cols_for_width(width, advance)
}

#[uniffi::export]
pub fn wrap_diff_lines(lines: Vec<DiffLine>, cols: u32) -> Vec<WrappedDiffLine> {
    diff::wrap_diff_lines(&lines, cols)
}

#[uniffi::export]
pub fn wrap_sbs_rows(rows: Vec<SideBySideRow>, old_cols: u32, new_cols: u32) -> Vec<WrappedSbsRow> {
    diff::wrap_sbs_rows(&rows, old_cols, new_cols)
}

#[uniffi::export]
pub fn conflict_display_text(kind: ConflictLineKind, raw: String) -> Option<String> {
    diff::conflict_display_text(kind, &raw)
}

#[uniffi::export]
pub fn diff_display_lines(lines: Vec<DiffLine>) -> Vec<DiffLine> {
    diff::build_diff_display_lines(&lines)
}

#[uniffi::export]
pub fn change_groups(lines: Vec<DiffLine>) -> Vec<ChangeGroup> {
    diff::change_groups(&lines)
}

#[uniffi::export]
pub fn highlight_file_lines(path: String, content: String) -> Vec<Vec<DiffSpan>> {
    diff::highlight_file(&path, &content)
}

// `visual_index_for_*` stay Rust-only — exporting would copy the full wrapped Vec across FFI per lookup.
