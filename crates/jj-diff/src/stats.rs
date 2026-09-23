use super::line_diff::{LineOp, line_diff};

/// Insertion/deletion counts matching the rendered diff rows, including the active whitespace mode.
pub fn count_changed_lines(old: &str, new: &str, ignore_whitespace: bool) -> (u32, u32) {
    let mut insertions = 0;
    let mut deletions = 0;
    for op in line_diff(old, new, ignore_whitespace) {
        match op {
            LineOp::Add => insertions += 1,
            LineOp::Remove => deletions += 1,
            LineOp::Equal => {}
        }
    }
    (insertions, deletions)
}
