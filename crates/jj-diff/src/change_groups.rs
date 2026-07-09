use super::types::{ChangeGroup, DiffLine, DiffSide, DiffSpanStyle};

/// The canonical group index used by review marks and notes across SwiftUI and CLI surfaces.
pub fn change_groups(lines: &[DiffLine]) -> Vec<ChangeGroup> {
    change_group_ranges(lines)
        .into_iter()
        .map(|(index, start, end)| build_change_group(index, start, end, lines))
        .collect()
}

/// Scans ranges first and materializes only the matching group's payload, so reconciling a note does not clone every changed line's text.
pub fn change_group_for_anchor(
    lines: &[DiffLine],
    side: DiffSide,
    line_number: u32,
    anchor_excerpt: &str,
) -> Option<ChangeGroup> {
    change_group_ranges(lines)
        .into_iter()
        .find(|(_, start, end)| {
            lines[*start..=*end]
                .iter()
                .any(|line| line_matches_anchor(line, side, line_number, anchor_excerpt))
        })
        .map(|(index, start, end)| build_change_group(index, start, end, lines))
}

/// 0-based inclusive (index, start, end) spans of contiguous changed lines.
fn change_group_ranges(lines: &[DiffLine]) -> Vec<(u32, usize, usize)> {
    let mut ranges: Vec<(u32, usize, usize)> = Vec::new();
    let mut start: Option<usize> = None;

    for (line_index, line) in lines.iter().enumerate() {
        if line.is_changed() {
            if start.is_none() {
                start = Some(line_index);
            }
        } else if let Some(start_index) = start.take() {
            ranges.push((ranges.len() as u32, start_index, line_index - 1));
        }
    }

    if let Some(start_index) = start {
        ranges.push((
            ranges.len() as u32,
            start_index,
            lines.len().saturating_sub(1),
        ));
    }

    ranges
}

fn build_change_group(
    index: u32,
    start_index: usize,
    end_index: usize,
    lines: &[DiffLine],
) -> ChangeGroup {
    let group_lines = &lines[start_index..=end_index];
    let anchor = group_lines
        .iter()
        .find_map(anchor_for_line)
        .expect("change group must contain a changed line with a line number");

    ChangeGroup {
        index,
        start_line: (start_index + 1) as u32,
        end_line: (end_index + 1) as u32,
        anchor_side: anchor.0,
        anchor_line: anchor.1,
        anchor_excerpt: anchor.2,
        anchor_context: group_lines.iter().map(DiffLine::text).collect(),
    }
}

fn anchor_for_line(line: &DiffLine) -> Option<(DiffSide, u32, String)> {
    match line.style {
        DiffSpanStyle::Removed => line
            .old_line_no
            .map(|line_no| (DiffSide::Old, line_no, line.text())),
        DiffSpanStyle::Added => line
            .new_line_no
            .map(|line_no| (DiffSide::New, line_no, line.text())),
        _ => None,
    }
}

fn line_matches_anchor(
    line: &DiffLine,
    side: DiffSide,
    line_number: u32,
    anchor_excerpt: &str,
) -> bool {
    match (side, line.style) {
        (DiffSide::Old, DiffSpanStyle::Removed) => {
            line.old_line_no == Some(line_number) && line.text() == anchor_excerpt
        }
        (DiffSide::New, DiffSpanStyle::Added) => {
            line.new_line_no == Some(line_number) && line.text() == anchor_excerpt
        }
        _ => false,
    }
}
