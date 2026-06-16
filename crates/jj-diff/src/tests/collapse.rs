use super::*;

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
    }
}

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
fn collapse_keeps_whole_committed_conflict_block() {
    // A committed conflict shows as unchanged Context markers; a change just
    // before a long conflict must not keep the `<<<<<<<` Start while collapsing
    // its matching `>>>>>>>` End into an unterminated block.
    let mut lines = vec![
        ctx_line("alpha"),
        change_line("beta-edited", DiffSpanStyle::Added),
        ctx_line("gamma"),
        ctx_line("delta"),
        ctx_line("<<<<<<< Conflict 1 of 1"),
        ctx_line("%%%%%%% Changes from base to side #1"),
        ctx_line("-base body 1"),
        ctx_line("-base body 2"),
        ctx_line("-base body 3"),
        ctx_line("+side body 1"),
        ctx_line("+side body 2"),
        ctx_line("+++++++ Contents of side #2"),
        ctx_line("other body 1"),
        ctx_line("other body 2"),
        ctx_line(">>>>>>> Conflict 1 of 1 ends"),
        ctx_line("epsilon"),
        ctx_line("zeta"),
        ctx_line("eta"),
        ctx_line("theta"),
        ctx_line("iota"),
    ];
    annotate_conflict_lines(&mut lines);

    let full = FileDiff {
        path: String::new(),
        language: String::new(),
        lines,
        whitespace_only_hidden: false,
    };
    let collapsed = collapse_context_with_mapping(&full);

    let starts = collapsed
        .diff
        .lines
        .iter()
        .filter(|l| l.conflict_kind == ConflictLineKind::Start)
        .count();
    let ends = collapsed
        .diff
        .lines
        .iter()
        .filter(|l| l.conflict_kind == ConflictLineKind::End)
        .count();
    assert_eq!(starts, 1, "the conflict Start must survive collapse");
    assert_eq!(
        ends, 1,
        "the matching conflict End must not be collapsed behind a separator"
    );

    // No separator may appear inside the conflict span, or build_diff_display_items
    // would swallow the rest of the diff into one bogus block.
    let start_ix = collapsed
        .diff
        .lines
        .iter()
        .position(|l| l.conflict_kind == ConflictLineKind::Start)
        .unwrap();
    let end_ix = collapsed
        .diff
        .lines
        .iter()
        .position(|l| l.conflict_kind == ConflictLineKind::End)
        .unwrap();
    assert!(
        collapsed.diff.lines[start_ix..=end_ix]
            .iter()
            .all(|l| l.style != DiffSpanStyle::Separator),
        "no separator may split a conflict block"
    );

    // The display layer must produce exactly one terminated conflict block.
    let blocks = build_diff_display_items(&collapsed.diff.lines)
        .into_iter()
        .filter(|item| matches!(item, DiffDisplayItem::ConflictBlock { .. }))
        .count();
    assert_eq!(blocks, 1);
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
