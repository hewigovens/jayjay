use super::*;

#[test]
fn context_collapsing() {
    let old_lines: Vec<String> = (1..=20).map(|i| format!("line {i}")).collect();
    let mut new_lines = old_lines.clone();
    new_lines[9] = "CHANGED".to_string();

    let old = old_lines.join("\n") + "\n";
    let new = new_lines.join("\n") + "\n";
    let diff = compute_file_diff("test.txt", &old, &new, false);
    assert!(
        diff.lines
            .iter()
            .any(|l| l.style == DiffSpanStyle::Separator),
        "Should have separator lines for collapsed context"
    );
}

#[test]
fn context_collapsing_keeps_tiny_gap_between_hunks() {
    let old_lines: Vec<String> = (1..=14).map(|i| format!("line {i}")).collect();
    let mut new_lines = old_lines.clone();
    new_lines[3] = "CHANGED 4".to_string();
    new_lines[11] = "CHANGED 12".to_string();

    let old = old_lines.join("\n") + "\n";
    let new = new_lines.join("\n") + "\n";
    let full = compute_file_diff_full("test.txt", &old, &new, false);
    let collapsed = collapse_context_with_mapping(&full);

    assert_eq!(
        collapsed.diff.lines.len(),
        full.lines.len(),
        "a one-line context gap is clearer inline than behind a separator"
    );
    assert!(
        collapsed
            .diff
            .lines
            .iter()
            .all(|l| l.style != DiffSpanStyle::Separator),
        "tiny context gaps should not be collapsed"
    );
}

#[test]
fn collapse_context_with_mapping_preserves_changed_lines() {
    let old_lines: Vec<String> = (1..=20).map(|i| format!("line {i}")).collect();
    let mut new_lines = old_lines.clone();
    new_lines[9] = "CHANGED".to_string();

    let old = old_lines.join("\n") + "\n";
    let new = new_lines.join("\n") + "\n";
    let full = compute_file_diff_full("test.txt", &old, &new, false);
    let collapsed = collapse_context_with_mapping(&full);

    assert!(
        collapsed.diff.lines.len() < full.lines.len(),
        "collapsed ({}) should have fewer lines than full ({})",
        collapsed.diff.lines.len(),
        full.lines.len()
    );
    assert!(
        collapsed
            .diff
            .lines
            .iter()
            .any(|l| l.style == DiffSpanStyle::Separator),
        "should have separator lines"
    );

    let changed: Vec<_> = collapsed
        .diff
        .lines
        .iter()
        .filter(|l| l.style == DiffSpanStyle::Removed || l.style == DiffSpanStyle::Added)
        .collect();
    assert_eq!(changed.len(), 2, "should preserve removed + added lines");

    let non_separator_count = collapsed
        .diff
        .lines
        .iter()
        .filter(|l| l.style != DiffSpanStyle::Separator)
        .count();
    assert_eq!(collapsed.display_to_full.len(), non_separator_count);
    for m in &collapsed.display_to_full {
        assert!((m.full_line as usize) <= full.lines.len());
    }
}

#[test]
fn collapse_with_mapping_small_diff_no_collapse() {
    let diff = compute_file_diff_full("test.txt", "a\nb\nc\n", "a\nX\nc\n", false);
    let collapsed = collapse_context_with_mapping(&diff);

    assert_eq!(collapsed.diff.lines.len(), diff.lines.len());
    assert!(
        collapsed
            .diff
            .lines
            .iter()
            .all(|l| l.style != DiffSpanStyle::Separator)
    );
}
