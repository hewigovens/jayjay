use super::*;

pub(super) fn three_gap_diff() -> (FileDiff, String, String) {
    let old_lines: Vec<String> = (1..=100).map(|line| format!("line {line}")).collect();
    let mut new_lines = old_lines.clone();
    new_lines[19] = "changed 20".to_owned();
    new_lines[79] = "changed 80".to_owned();
    let old = old_lines.join("\n") + "\n";
    let new = new_lines.join("\n") + "\n";
    (compute_file_diff("sample.rs", &old, &new, false), old, new)
}

pub(super) fn regions(diff: &FileDiff) -> Vec<ContextRegion> {
    diff.lines
        .iter()
        .filter_map(|line| line.context_region)
        .collect()
}
