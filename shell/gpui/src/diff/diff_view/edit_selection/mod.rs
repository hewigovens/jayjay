//! Maps a display-line selection in the collapsed diff to full-diff `DiffEditRange`s.

use std::collections::HashSet;
use std::ops::RangeInclusive;

use jayjay_core::DiffEditRange;
use jayjay_core::diff::{
    ChangeGroup, DiffLine, DiffSpanStyle, change_groups, compute_file_diff_full,
};

/// Maps 0-based display-line indices in the collapsed diff to 1-based `DiffEditRange`s in the full diff by matching (style, old_line_no, new_line_no) — a stable key because collapsing only elides unchanged context and never renumbers changed lines.
pub(crate) fn display_range_to_diff_edit_range(
    path: &str,
    collapsed_lines: &[DiffLine],
    old_content: &str,
    new_content: &str,
    ignore_whitespace: bool,
    display_range: RangeInclusive<usize>,
) -> Vec<DiffEditRange> {
    let selected_keys: HashSet<ChangedLineKey> = display_range
        .filter_map(|ix| collapsed_lines.get(ix))
        .filter_map(ChangedLineKey::for_line)
        .collect();
    if selected_keys.is_empty() {
        return Vec::new();
    }

    let full = compute_file_diff_full(path, old_content, new_content, ignore_whitespace);
    let full_line_numbers: Vec<u32> = full
        .lines
        .iter()
        .enumerate()
        .filter(|(_, line)| {
            ChangedLineKey::for_line(line).is_some_and(|key| selected_keys.contains(&key))
        })
        .map(|(ix, _)| (ix + 1) as u32)
        .collect();

    collapse_into_ranges(&full_line_numbers)
}

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

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
struct ChangedLineKey {
    added: bool,
    old_line_no: Option<u32>,
    new_line_no: Option<u32>,
}

impl ChangedLineKey {
    fn for_line(line: &DiffLine) -> Option<Self> {
        match line.style {
            DiffSpanStyle::Added => Some(Self {
                added: true,
                old_line_no: line.old_line_no,
                new_line_no: line.new_line_no,
            }),
            DiffSpanStyle::Removed => Some(Self {
                added: false,
                old_line_no: line.old_line_no,
                new_line_no: line.new_line_no,
            }),
            _ => None,
        }
    }
}

fn collapse_into_ranges(sorted_line_numbers: &[u32]) -> Vec<DiffEditRange> {
    let mut ranges = Vec::new();
    let mut iter = sorted_line_numbers.iter().copied();
    let Some(first) = iter.next() else {
        return ranges;
    };
    let mut start = first;
    let mut previous = first;
    for line_no in iter {
        if line_no == previous + 1 {
            previous = line_no;
            continue;
        }
        ranges.push(DiffEditRange {
            start_line: start,
            end_line: previous,
        });
        start = line_no;
        previous = line_no;
    }
    ranges.push(DiffEditRange {
        start_line: start,
        end_line: previous,
    });
    ranges
}

#[cfg(test)]
mod tests;
