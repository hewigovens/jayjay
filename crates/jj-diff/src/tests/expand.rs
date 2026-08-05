use std::collections::HashSet;

use super::fixtures::{regions, three_gap_diff};
use super::*;

#[test]
fn collapse_assigns_stable_regions_to_top_middle_and_end_gaps() {
    let (diff, _, _) = three_gap_diff();
    let regions = regions(&diff);

    assert_eq!(regions.len(), 3);
    assert_eq!(regions[0].old_start_line, 1);
    assert_eq!(regions[0].new_start_line, 1);
    assert_eq!(regions[2].old_start_line + regions[2].line_count, 101);
    assert_eq!(regions[2].new_start_line + regions[2].line_count, 101);
    assert_eq!(
        regions
            .iter()
            .map(|region| region.id)
            .collect::<HashSet<_>>()
            .len(),
        regions.len()
    );
}

#[test]
fn show_more_reveals_suffix_below_stable_separator_repeatedly() {
    let (diff, old, new) = three_gap_diff();
    let middle = regions(&diff)[1];
    assert!(middle.line_count > 20);
    let mut expandable = ExpandableDiff::new(diff, old, new);

    let first = expandable
        .expand(middle.id, ContextExpansion::ShowMore { line_count: 10 })
        .unwrap();
    let separator_index = first
        .diff
        .lines
        .iter()
        .position(|line| {
            line.context_region
                .is_some_and(|region| region.id == middle.id)
        })
        .unwrap();
    let remaining = first.diff.lines[separator_index].context_region.unwrap();
    assert_eq!(remaining.id, middle.id);
    assert_eq!(remaining.line_count, middle.line_count - 10);
    assert_eq!(first.inserted.start, separator_index as u32 + 1);
    assert_eq!(first.inserted.count, 10);
    assert_eq!(
        first.diff.lines[separator_index + 1].new_line_no,
        Some(middle.new_start_line + middle.line_count - 10)
    );

    let second = expandable
        .expand(middle.id, ContextExpansion::ShowMore { line_count: 10 })
        .unwrap();
    let remaining = second
        .diff
        .lines
        .iter()
        .find_map(|line| line.context_region.filter(|region| region.id == middle.id))
        .unwrap();
    assert_eq!(remaining.id, middle.id);
    assert_eq!(remaining.line_count, middle.line_count - 20);
}

#[test]
fn show_all_removes_region_and_stale_id_is_typed_error() {
    let (diff, old, new) = three_gap_diff();
    let region = regions(&diff)[0];
    let mut expandable = ExpandableDiff::new(diff, old, new);

    let expanded = expandable
        .expand(region.id, ContextExpansion::ShowAll)
        .unwrap();
    assert_eq!(expanded.inserted.start, 0);
    assert_eq!(expanded.inserted.count, region.line_count);
    assert!(
        expanded
            .diff
            .lines
            .iter()
            .all(|line| line.context_region.is_none_or(|item| item.id != region.id))
    );
    assert_eq!(
        expandable
            .expand(region.id, ContextExpansion::ShowAll)
            .unwrap_err(),
        ContextExpansionError::UnknownRegion {
            region_id: region.id
        }
    );
}

#[test]
fn show_more_on_a_trailing_region_reveals_its_prefix_and_moves_the_separator_below() {
    let (diff, old, new) = three_gap_diff();
    let trailing = regions(&diff)[2];
    let mut expandable = ExpandableDiff::new(diff, old, new);

    let expanded = expandable
        .expand(trailing.id, ContextExpansion::ShowMore { line_count: 10 })
        .unwrap();

    assert_eq!(expanded.inserted.count, 10);
    let first_revealed = &expanded.diff.lines[expanded.inserted.start as usize];
    assert_eq!(first_revealed.new_line_no, Some(trailing.new_start_line));
    let separator = expanded
        .diff
        .lines
        .iter()
        .find_map(|line| {
            line.context_region
                .filter(|region| region.id == trailing.id)
        })
        .unwrap();
    assert_eq!(separator.line_count, trailing.line_count - 10);
    assert_eq!(separator.new_start_line, trailing.new_start_line + 10);
    assert!(
        expanded
            .diff
            .lines
            .last()
            .unwrap()
            .context_region
            .is_some_and(|region| region.id == trailing.id),
        "the reduced separator stays below the revealed prefix at the end of the diff"
    );
}

#[test]
fn show_more_exceeding_region_reveals_everything_and_removes_separator() {
    let (diff, old, new) = three_gap_diff();
    let region = regions(&diff)[0];
    let before_len = diff.lines.len();
    let mut expandable = ExpandableDiff::new(diff, old, new);

    let expanded = expandable
        .expand(
            region.id,
            ContextExpansion::ShowMore {
                line_count: region.line_count + 10,
            },
        )
        .unwrap();

    assert_eq!(expanded.inserted.start, 0);
    assert_eq!(expanded.inserted.count, region.line_count);
    assert_eq!(
        expanded.diff.lines.len(),
        before_len - 1 + region.line_count as usize
    );
    assert!(
        expanded
            .diff
            .lines
            .iter()
            .all(|line| line.context_region.is_none_or(|item| item.id != region.id))
    );
    assert_eq!(
        expanded.diff.lines[0].new_line_no,
        Some(region.new_start_line)
    );
    assert_eq!(
        expanded.diff.lines[region.line_count as usize - 1].new_line_no,
        Some(region.new_start_line + region.line_count - 1)
    );
}

#[test]
fn expand_all_reveals_every_region_and_is_idempotent() {
    let (diff, old, new) = three_gap_diff();
    assert!(regions(&diff).len() > 1);
    let mut expandable = ExpandableDiff::new(diff, old, new);

    let expanded = expandable.expand_all().unwrap();

    assert!(
        expanded
            .diff
            .lines
            .iter()
            .all(|line| line.context_region.is_none())
    );
    assert_eq!(expanded.diff.lines.len(), 102);
    let again = expandable.expand_all().unwrap();
    assert_eq!(again.inserted.count, 0);
}

#[test]
fn zero_show_more_is_rejected_without_changing_diff() {
    let (diff, old, new) = three_gap_diff();
    let region = regions(&diff)[1];
    let original_len = diff.lines.len();
    let mut expandable = ExpandableDiff::new(diff, old, new);

    assert_eq!(
        expandable
            .expand(region.id, ContextExpansion::ShowMore { line_count: 0 })
            .unwrap_err(),
        ContextExpansionError::InvalidLineCount
    );
    assert_eq!(expandable.diff().lines.len(), original_len);
}
