use std::collections::HashMap;

use super::moved_lines::MovedLines;
use crate::types::{DiffLine, DiffSpanStyle, FileDiff, LineSpan};

const MIN_ROWS_APART: usize = 3;
const MIN_SHARED_TEXT: usize = 8;

impl FileDiff {
    pub fn detect_moves(&mut self, old: &str, new: &str) {
        if !self.may_have_moves() {
            return;
        }
        let Some(found) = MovedLines::find(self, old, new) else {
            return;
        };
        let moves = self.moves_between_hunks(&found);
        for line in self.lines.iter_mut().filter(|line| line.is_changed()) {
            line.moved = moves.iter().find_map(|block| block.row(line));
        }
    }

    /// A move needs a removed and an added row with the same tokens in different hunks; checking their text first skips parsing nearly every file.
    pub(super) fn may_have_moves(&self) -> bool {
        let mut removed: HashMap<String, (usize, usize)> = HashMap::new();
        let mut added = Vec::new();
        for (hunk, _, line) in self.changed_rows() {
            let text: String = line
                .spans
                .iter()
                .flat_map(|span| span.text.chars())
                .filter(|c| !c.is_whitespace())
                .collect();
            if text.len() < MIN_SHARED_TEXT {
                continue;
            }
            if line.style == DiffSpanStyle::Removed {
                removed.entry(text).or_insert((hunk, hunk)).1 = hunk;
            } else {
                added.push((text, hunk));
            }
        }
        added.iter().any(|(text, hunk)| {
            removed
                .get(text)
                .is_some_and(|&(first, last)| first != *hunk || last != *hunk)
        })
    }

    /// Old → new line, zero-based, for every line the displayed diff keeps.
    pub(super) fn kept_rows(&self) -> HashMap<usize, usize> {
        let mut rows = HashMap::new();
        for line in &self.lines {
            if let Some(region) = line.context_region {
                for offset in 0..region.line_count {
                    rows.insert(
                        (region.old_start_line + offset - 1) as usize,
                        (region.new_start_line + offset - 1) as usize,
                    );
                }
            } else if let (DiffSpanStyle::Context, Some(old), Some(new)) =
                (line.style, line.old_line_no, line.new_line_no)
            {
                rows.insert(old as usize - 1, new as usize - 1);
            }
        }
        rows
    }

    /// Changed lines as (hunk, row index, line).
    fn changed_rows(&self) -> impl Iterator<Item = (usize, usize, &DiffLine)> {
        let mut hunk = 0;
        self.lines
            .iter()
            .enumerate()
            .filter_map(move |(row, line)| {
                if row > 0 && line.is_changed() != self.lines[row - 1].is_changed() {
                    hunk += 1;
                }
                line.is_changed().then_some((hunk, row, line))
            })
    }

    /// A nearby reorder is already in view, and the line diff may pair a moved block's closing brace as context.
    fn moves_between_hunks(&self, moved: &[MovedLines]) -> Vec<MovedLines> {
        // Line number → (hunk, row index) of each removed and added row.
        let (mut removed, mut added) = (HashMap::new(), HashMap::new());
        for (hunk, row, line) in self.changed_rows() {
            match (line.style, line.old_line_no, line.new_line_no) {
                (DiffSpanStyle::Removed, Some(old), _) => removed.insert(old, (hunk, row)),
                (DiffSpanStyle::Added, _, Some(new)) => added.insert(new, (hunk, row)),
                _ => None,
            };
        }
        let trimmed = |rows: &HashMap<u32, (usize, usize)>, span: LineSpan| {
            let mut lines =
                (span.start..span.start + span.count).filter(|line| rows.contains_key(line));
            let first = lines.next()?;
            let last = lines.next_back().unwrap_or(first);
            let (hunk, first_row) = rows[&first];
            let same_hunk =
                (first..=last).all(|line| rows.get(&line).is_some_and(|&(h, _)| h == hunk));
            same_hunk.then_some((
                LineSpan {
                    start: first,
                    count: last - first + 1,
                },
                hunk,
                first_row..rows[&last].1 + 1,
            ))
        };
        moved
            .iter()
            .filter_map(|block| {
                let (old_lines, old_hunk, old_rows) = trimmed(&removed, block.old_lines)?;
                let (new_lines, new_hunk, new_rows) = trimmed(&added, block.new_lines)?;
                let rows_apart = if old_rows.start < new_rows.start {
                    new_rows.start.saturating_sub(old_rows.end)
                } else {
                    old_rows.start.saturating_sub(new_rows.end)
                };
                (old_hunk != new_hunk && rows_apart >= MIN_ROWS_APART).then_some(MovedLines {
                    old_lines,
                    new_lines,
                })
            })
            .collect()
    }
}
