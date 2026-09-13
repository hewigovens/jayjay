use super::HunkType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffEditDestination {
    RemoveFromSource,
    MoveToWorkingCopy,
    NewChild,
    NewParallel,
}

#[derive(Debug, Clone)]
pub struct DiffEditRange {
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Debug, Clone)]
pub struct DiffEditFileSelection {
    pub path: String,
    pub old_path: Option<String>,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub hunk_type: HunkType,
    pub line_ranges: Vec<DiffEditRange>,
}

/// Collapses sorted, deduplicated line numbers into the contiguous ranges the tree rewrite applies.
pub fn diff_edit_ranges(mut lines: Vec<u32>) -> Vec<DiffEditRange> {
    lines.sort_unstable();
    lines.dedup();
    let mut ranges: Vec<DiffEditRange> = Vec::new();
    for line in lines {
        match ranges.last_mut() {
            Some(range) if range.end_line.checked_add(1) == Some(line) => range.end_line = line,
            _ => ranges.push(DiffEditRange {
                start_line: line,
                end_line: line,
            }),
        }
    }
    ranges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coalesces_sorted_unique_line_ranges() {
        let ranges = diff_edit_ranges(vec![5, 2, 3, 3, 9]);
        assert_eq!(
            ranges
                .iter()
                .map(|range| (range.start_line, range.end_line))
                .collect::<Vec<_>>(),
            vec![(2, 3), (5, 5), (9, 9)]
        );
    }
}
