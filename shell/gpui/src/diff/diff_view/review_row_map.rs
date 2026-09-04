use jayjay_core::diff::{FileDiff, build_diff_display_lines, change_groups, sbs_line_to_row};

pub(crate) struct ReviewRowMap {
    pub(crate) group_count: usize,
    pub(crate) unified_line_groups: Vec<Option<u32>>,
    pub(crate) side_by_side_row_groups: Vec<Option<u32>>,
}

impl ReviewRowMap {
    pub(super) fn new(diff: &FileDiff) -> Self {
        let display_lines = build_diff_display_lines(&diff.lines);
        let groups = change_groups(&display_lines);
        let mut unified_line_groups = vec![None; display_lines.len()];
        for group in &groups {
            for line in group.start_line..=group.end_line {
                unified_line_groups[line as usize - 1] = Some(group.index);
            }
        }
        let display_to_sbs = sbs_line_to_row(&diff.lines);
        let row_count = display_to_sbs
            .iter()
            .map(|row| *row as usize + 1)
            .max()
            .unwrap_or(0);
        let mut side_by_side_row_groups = vec![None; row_count];
        for (line, group) in unified_line_groups.iter().enumerate() {
            if let (Some(group), Some(row)) = (group, display_to_sbs.get(line)) {
                side_by_side_row_groups[*row as usize] = Some(*group);
            }
        }
        Self {
            group_count: groups.len(),
            unified_line_groups,
            side_by_side_row_groups,
        }
    }
}
