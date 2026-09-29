use super::diff_render_rows::DiffRenderRow;

/// Scroll targets must map through this — note rows shift row indices past wrapped-line indices.
pub fn row_index_for_line(rows: &[DiffRenderRow], wrapped_line_ix: usize) -> usize {
    rows.iter()
        .position(|row| matches!(row, DiffRenderRow::Line(ix) if *ix == wrapped_line_ix))
        .unwrap_or(wrapped_line_ix)
}
