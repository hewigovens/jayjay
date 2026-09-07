use super::*;

use crate::conflicts::{annotate_conflict_lines, build_diff_display_items};

fn ctx_line(text: &str) -> DiffLine {
    change_line(text, DiffSpanStyle::Context)
}

fn change_line(text: &str, style: DiffSpanStyle) -> DiffLine {
    DiffLine {
        old_line_no: Some(1),
        new_line_no: Some(1),
        style,
        spans: vec![DiffSpan {
            text: text.to_owned(),
            style,
            token: SyntaxToken::Plain,
        }],
        conflict_kind: ConflictLineKind::None,
        no_eof_newline: false,
        context_region: None,
    }
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
fn collapse_keeps_a_committed_conflict_block_whole_while_collapsing_around_it() {
    let mut lines: Vec<DiffLine> = (1..=80)
        .map(|line| ctx_line(&format!("line {line}")))
        .collect();
    lines[4] = change_line("changed", DiffSpanStyle::Added);
    let conflict = [
        "<<<<<<< Conflict 1 of 1",
        "%%%%%%% Changes from base to side #1",
        "-base",
        "+++++++ Contents of side #2",
        "+other",
        ">>>>>>> Conflict 1 of 1 ends",
    ];
    for (offset, text) in conflict.into_iter().enumerate() {
        lines[50 + offset] = ctx_line(text);
    }
    annotate_conflict_lines(&mut lines);

    let collapsed = collapse_context_with_mapping(&FileDiff {
        path: "conflict.txt".to_owned(),
        language: "plaintext".to_owned(),
        lines,
        whitespace_only_hidden: false,
    });
    let lines = &collapsed.diff.lines;
    let start = lines
        .iter()
        .position(|line| line.conflict_kind == ConflictLineKind::Start)
        .unwrap();
    let end = lines
        .iter()
        .position(|line| line.conflict_kind == ConflictLineKind::End)
        .unwrap();
    assert_eq!(end - start, conflict.len() - 1);
    assert!(
        lines[start..=end]
            .iter()
            .all(|line| line.style != DiffSpanStyle::Separator && line.context_region.is_none())
    );
    let separators: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.style == DiffSpanStyle::Separator)
        .map(|(ix, _)| ix)
        .collect();
    assert_eq!(
        separators,
        vec![start - 1, end + 1],
        "context on both sides of the block still collapses"
    );
    let blocks = build_diff_display_items(lines)
        .into_iter()
        .filter(|item| matches!(item, DiffDisplayItem::ConflictBlock { .. }))
        .count();
    assert_eq!(blocks, 1);
}

#[test]
fn unmatched_conflict_start_marker_does_not_pin_the_tail_visible() {
    let mut old_lines: Vec<String> = (1..=80).map(|i| format!("line {i}")).collect();
    old_lines[10] = "<<<<<<< quoted marker in ordinary text".to_owned();
    let mut new_lines = old_lines.clone();
    new_lines[0] = "changed".to_owned();
    let old = old_lines.join("\n") + "\n";
    let new = new_lines.join("\n") + "\n";

    let diff = compute_file_diff("sample.txt", &old, &new, false);

    let widest_region = diff
        .lines
        .iter()
        .filter_map(|line| line.context_region)
        .map(|region| region.line_count)
        .max()
        .unwrap_or(0);
    assert!(
        widest_region > 60,
        "the tail collapses past the unmatched marker (widest hidden run: {widest_region} lines)"
    );
}
