use super::line_diff::{LineOp, line_diff};

/// Insertion/deletion counts matching the rendered diff rows, including the active whitespace mode.
pub fn count_changed_lines(old: &str, new: &str, ignore_whitespace: bool) -> (u32, u32) {
    if old.is_empty() && new.is_empty() {
        return (0, 0);
    }
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    let ops = line_diff(&old_lines, &new_lines, ignore_whitespace);
    let mut insertions = 0;
    let mut deletions = 0;
    for op in &ops {
        match op {
            LineOp::Add => insertions += 1,
            LineOp::Remove => deletions += 1,
            LineOp::Equal => {}
        }
    }
    // The renderer splits a shared trailing context line into a removed/added pair when only EOF-newline presence differs; mirror that so counts match visible rows.
    let no_eof_old = !old.is_empty() && !old.ends_with('\n');
    let no_eof_new = !new.is_empty() && !new.ends_with('\n');
    if no_eof_old != no_eof_new && ops.last() == Some(&LineOp::Equal) {
        insertions += 1;
        deletions += 1;
    }
    (insertions, deletions)
}
