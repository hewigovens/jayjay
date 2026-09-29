use std::ops::RangeInclusive;

use jayjay_core::diff::{ChangeGroup, DiffLine, change_groups};

pub(crate) fn selection_covers_whole_change_group(
    display_lines: &[DiffLine],
    display_range: RangeInclusive<usize>,
) -> bool {
    let changed_ixs: Vec<usize> = display_range
        .clone()
        .filter(|&ix| display_lines.get(ix).is_some_and(DiffLine::is_changed))
        .collect();
    if changed_ixs.len() < 2 {
        return false;
    }
    let Some(group) = find_group_containing(display_lines, changed_ixs[0]) else {
        return false;
    };
    let group_range = group_display_range(&group);
    changed_ixs.iter().all(|ix| group_range.contains(ix))
        && *group_range.start() == *display_range.start()
        && *group_range.end() == *display_range.end()
}

fn find_group_containing(lines: &[DiffLine], ix: usize) -> Option<ChangeGroup> {
    change_groups(lines)
        .into_iter()
        .find(|group| group_display_range(group).contains(&ix))
}

/// `ChangeGroup::start_line`/`end_line` are 1-based; convert to the 0-based basis used by `display_range`.
fn group_display_range(group: &ChangeGroup) -> RangeInclusive<usize> {
    (group.start_line as usize - 1)..=(group.end_line as usize - 1)
}
