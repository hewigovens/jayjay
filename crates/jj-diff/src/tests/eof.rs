use super::*;

#[test]
fn eof_newline_added_synthesizes_visible_hunk() {
    let diff = compute_file_diff("test.txt", "a\nb", "a\nb\n", false);
    let removed = diff
        .lines
        .iter()
        .find(|l| l.style == DiffSpanStyle::Removed)
        .expect("should have a removed line for EOF newline change");
    let added = diff
        .lines
        .iter()
        .find(|l| l.style == DiffSpanStyle::Added)
        .expect("should have an added line for EOF newline change");

    assert!(removed.no_eof_newline);
    assert!(!added.no_eof_newline);
}

#[test]
fn eof_newline_removed_synthesizes_visible_hunk() {
    let diff = compute_file_diff("test.txt", "a\nb\n", "a\nb", false);
    let removed = diff
        .lines
        .iter()
        .find(|l| l.style == DiffSpanStyle::Removed)
        .expect("should have a removed line");
    let added = diff
        .lines
        .iter()
        .find(|l| l.style == DiffSpanStyle::Added)
        .expect("should have an added line");

    assert!(!removed.no_eof_newline);
    assert!(added.no_eof_newline);
}

#[test]
fn eof_newline_only_no_whitespace_flag() {
    let diff = compute_file_diff("test.txt", "a\nb", "a\nb\n", false);
    assert!(
        !diff.whitespace_only_hidden,
        "EOF-newline change should synthesize a visible hunk, not set whitespace flag"
    );
}

#[test]
fn eof_splits_shared_context_when_real_changes_exist() {
    let diff = compute_file_diff("test.txt", "a\nchanged\nc", "a\nfixed\nc\n", false);

    let tail_removed = diff
        .lines
        .iter()
        .rev()
        .find(|l| l.style == DiffSpanStyle::Removed && l.old_line_no == Some(3))
        .expect("trailing shared-context 'c' should split into a removed line");
    let tail_added = diff
        .lines
        .iter()
        .rev()
        .find(|l| l.style == DiffSpanStyle::Added && l.new_line_no == Some(3))
        .expect("trailing shared-context 'c' should split into an added line");
    assert!(tail_removed.no_eof_newline);
    assert!(!tail_added.no_eof_newline);

    let mid_removed = diff
        .lines
        .iter()
        .find(|l| l.style == DiffSpanStyle::Removed && l.old_line_no == Some(2))
        .expect("middle 'changed' removed line must still exist");
    assert!(!mid_removed.no_eof_newline);
}

#[test]
fn eof_marker_lands_on_side_without_its_own_op() {
    let diff = compute_file_diff("test.txt", "a", "a\nb\n", false);
    let last_old = diff
        .lines
        .iter()
        .rev()
        .find(|l| l.old_line_no.is_some())
        .expect("diff should contain a line representing old");
    assert!(last_old.no_eof_newline);
    for line in diff.lines.iter().filter(|l| l.new_line_no.is_some()) {
        assert!(!line.no_eof_newline);
    }
}

#[test]
fn eof_newline_marker_on_real_change() {
    let diff = compute_file_diff("test.txt", "a\nold", "a\nnew\n", false);
    let last_removed = diff
        .lines
        .iter()
        .rev()
        .find(|l| l.style == DiffSpanStyle::Removed)
        .expect("should have a removed line");
    let last_added = diff
        .lines
        .iter()
        .rev()
        .find(|l| l.style == DiffSpanStyle::Added)
        .expect("should have an added line");

    assert!(last_removed.no_eof_newline);
    assert!(!last_added.no_eof_newline);
}
