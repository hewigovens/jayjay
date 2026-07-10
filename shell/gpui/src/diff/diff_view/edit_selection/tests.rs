use super::*;
use jayjay_core::diff::compute_file_diff;

fn range(ranges: &[DiffEditRange], ix: usize) -> (u32, u32) {
    (ranges[ix].start_line, ranges[ix].end_line)
}

#[test]
fn empty_selection_yields_no_ranges() {
    let old = "a\nb\n";
    let new = "a\nx\n";
    let collapsed = compute_file_diff("f.txt", old, new, false);
    // Line 0 is unchanged context — selecting only it should select nothing.
    let ranges =
        display_range_to_diff_edit_range("f.txt", &collapsed.lines, old, new, false, 0..=0);
    assert!(ranges.is_empty());
}

#[test]
fn contiguous_added_and_removed_pair_collapses_to_one_range() {
    let old = "a\nb\nc\n";
    let new = "a\nx\nc\n";
    let collapsed = compute_file_diff("f.txt", old, new, false);
    let removed_ix = collapsed
        .lines
        .iter()
        .position(|l| l.style == DiffSpanStyle::Removed)
        .expect("removed line present");
    let added_ix = collapsed
        .lines
        .iter()
        .position(|l| l.style == DiffSpanStyle::Added)
        .expect("added line present");
    let lo = removed_ix.min(added_ix);
    let hi = removed_ix.max(added_ix);

    let ranges =
        display_range_to_diff_edit_range("f.txt", &collapsed.lines, old, new, false, lo..=hi);
    assert_eq!(ranges.len(), 1);
    let full = compute_file_diff_full("f.txt", old, new, false);
    let expected_lo = full
        .lines
        .iter()
        .position(|l| l.style == DiffSpanStyle::Removed)
        .unwrap() as u32
        + 1;
    let expected_hi = full
        .lines
        .iter()
        .position(|l| l.style == DiffSpanStyle::Added)
        .unwrap() as u32
        + 1;
    assert_eq!(range(&ranges, 0), (expected_lo, expected_hi));
}

// Regression test: once a separator collapses hidden context, display index no longer equals `full_line - 1`, so the mapping must go through line identity, not arithmetic.
#[test]
fn maps_collapsed_display_index_to_full_diff_line_when_context_is_hidden() {
    let mut old_lines: Vec<String> = (1..=30).map(|n| format!("line{n}")).collect();
    old_lines[4] = "old5".to_owned();
    old_lines[24] = "old25".to_owned();
    let mut new_lines = old_lines.clone();
    new_lines[4] = "new5".to_owned();
    new_lines[24] = "new25".to_owned();
    let old = old_lines.join("\n") + "\n";
    let new = new_lines.join("\n") + "\n";

    let collapsed = compute_file_diff("f.txt", &old, &new, false);
    assert!(
        collapsed
            .lines
            .iter()
            .any(|l| l.style == DiffSpanStyle::Separator),
        "the gap between the two edits must collapse for this test to be meaningful"
    );

    let display_ix = collapsed
        .lines
        .iter()
        .position(|l| l.style == DiffSpanStyle::Added && l.new_line_no == Some(25))
        .expect("second edit's added line is displayed");

    let full = compute_file_diff_full("f.txt", &old, &new, false);
    let expected_full_line = full
        .lines
        .iter()
        .position(|l| l.style == DiffSpanStyle::Added && l.new_line_no == Some(25))
        .unwrap() as u32
        + 1;
    assert_ne!(
        display_ix as u32 + 1,
        expected_full_line,
        "display index must diverge from the full-diff line for this regression test to mean anything"
    );

    let ranges = display_range_to_diff_edit_range(
        "f.txt",
        &collapsed.lines,
        &old,
        &new,
        false,
        display_ix..=display_ix,
    );
    assert_eq!(range(&ranges, 0), (expected_full_line, expected_full_line));
}

#[test]
fn two_selected_pairs_far_apart_yield_two_ranges() {
    let old = "a\nb\nc\nd\ne\nf\ng\nh\ni\nj\nk\n";
    let new = "a\nx\nc\nd\ne\nf\ng\nh\ni\ny\nk\n";
    let collapsed = compute_file_diff("f.txt", old, new, false);
    let last_ix = collapsed.lines.len() - 1;

    let ranges =
        display_range_to_diff_edit_range("f.txt", &collapsed.lines, old, new, false, 0..=last_ix);
    assert_eq!(ranges.len(), 2, "the two edits are not adjacent");
}

#[test]
fn selection_covers_whole_change_group_requires_more_than_one_changed_line() {
    let old = "a\nb\nc\n";
    let new = "a\nx\nc\n";
    let collapsed = compute_file_diff("f.txt", old, new, false);
    let removed_ix = collapsed
        .lines
        .iter()
        .position(|l| l.style == DiffSpanStyle::Removed)
        .unwrap();
    assert!(!selection_covers_whole_change_group(
        &collapsed.lines,
        removed_ix..=removed_ix
    ));
}

#[test]
fn selection_covers_whole_change_group_true_for_exact_group_bounds() {
    let old = "a\nb\nc\ne\n";
    let new = "a\nx\ny\ne\n";
    let collapsed = compute_file_diff("f.txt", old, new, false);
    let lo = collapsed
        .lines
        .iter()
        .position(DiffLine::is_changed)
        .unwrap();
    let hi = collapsed
        .lines
        .iter()
        .rposition(DiffLine::is_changed)
        .unwrap();
    assert!(selection_covers_whole_change_group(
        &collapsed.lines,
        lo..=hi
    ));
    // A range that includes an extra context line no longer matches the group exactly.
    assert!(!selection_covers_whole_change_group(
        &collapsed.lines,
        0..=hi
    ));
}
