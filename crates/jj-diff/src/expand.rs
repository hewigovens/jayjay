use std::sync::Arc;

use crate::compute::should_skip_highlight;
use crate::render_highlights::{SideHighlights, apply_side_highlights, plain_spans};
use crate::types::{
    ConflictLineKind, ContextExpansion, ContextExpansionError, ContextExpansionResult,
    ContextRegion, DiffLine, DiffSpanStyle, FileDiff, LineIndex, LineSpan,
};

/// Selected-file-only state for expanding collapsed context without retaining or rebuilding a full diff.
pub struct ExpandableDiff {
    diff: FileDiff,
    old_content: Arc<str>,
    new_content: Arc<str>,
    old_line_index: LineIndex,
    new_line_index: LineIndex,
    highlights: Option<(SideHighlights, SideHighlights)>,
}

impl ExpandableDiff {
    pub fn new(diff: FileDiff, old_content: String, new_content: String) -> Self {
        Self::from_shared(diff, Arc::from(old_content), Arc::from(new_content))
    }

    pub fn from_shared(diff: FileDiff, old_content: Arc<str>, new_content: Arc<str>) -> Self {
        let old_line_index = LineIndex::from_text(&old_content);
        let new_line_index = LineIndex::from_text(&new_content);
        Self {
            diff,
            old_content,
            new_content,
            old_line_index,
            new_line_index,
            highlights: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn diff(&self) -> &FileDiff {
        &self.diff
    }

    pub fn expand(
        &mut self,
        region_id: u32,
        expansion: ContextExpansion,
    ) -> Result<ContextExpansionResult, ContextExpansionError> {
        let inserted = self.expand_in_place(region_id, expansion)?;
        Ok(ContextExpansionResult {
            diff: self.diff.clone(),
            inserted,
        })
    }

    fn expand_in_place(
        &mut self,
        region_id: u32,
        expansion: ContextExpansion,
    ) -> Result<LineSpan, ContextExpansionError> {
        let separator_index = self
            .diff
            .lines
            .iter()
            .position(|line| {
                line.style == DiffSpanStyle::Separator
                    && line
                        .context_region
                        .is_some_and(|region| region.id == region_id)
            })
            .ok_or(ContextExpansionError::UnknownRegion { region_id })?;
        let region = self.diff.lines[separator_index]
            .context_region
            .ok_or(ContextExpansionError::UnknownRegion { region_id })?;

        let reveal_count = match expansion {
            ContextExpansion::ShowMore { line_count: 0 } => {
                return Err(ContextExpansionError::InvalidLineCount);
            }
            ContextExpansion::ShowMore { line_count } => line_count.min(region.line_count),
            ContextExpansion::ShowAll => region.line_count,
        };
        if reveal_count == 0 {
            return Err(ContextExpansionError::InvalidRegion { region_id });
        }

        let remaining_count = region.line_count - reveal_count;
        // A trailing region has no visible content below it, so revealing its suffix would show file-end lines detached from the hunk above; reveal its prefix and move the separator below.
        let is_trailing = separator_index == self.diff.lines.len() - 1;
        let reveal_offset = match expansion {
            ContextExpansion::ShowMore { .. } if !is_trailing => remaining_count,
            _ => 0,
        };
        let old_start_line = region
            .old_start_line
            .checked_add(reveal_offset)
            .ok_or(ContextExpansionError::InvalidRegion { region_id })?;
        let new_start_line = region
            .new_start_line
            .checked_add(reveal_offset)
            .ok_or(ContextExpansionError::InvalidRegion { region_id })?;
        let revealed =
            self.context_lines(region_id, old_start_line, new_start_line, reveal_count)?;

        let inserted_line_start = if remaining_count == 0 {
            self.diff
                .lines
                .splice(separator_index..=separator_index, revealed);
            separator_index
        } else if is_trailing {
            let moved = crate::context::separator_line(ContextRegion {
                old_start_line: old_start_line
                    .checked_add(reveal_count)
                    .ok_or(ContextExpansionError::InvalidRegion { region_id })?,
                new_start_line: new_start_line
                    .checked_add(reveal_count)
                    .ok_or(ContextExpansionError::InvalidRegion { region_id })?,
                line_count: remaining_count,
                ..region
            });
            self.diff.lines.splice(
                separator_index..=separator_index,
                revealed.into_iter().chain([moved]),
            );
            separator_index
        } else {
            self.diff.lines[separator_index] = crate::context::separator_line(ContextRegion {
                line_count: remaining_count,
                ..region
            });
            self.diff
                .lines
                .splice(separator_index + 1..separator_index + 1, revealed);
            separator_index + 1
        };
        self.apply_full_highlights(
            inserted_line_start..inserted_line_start + reveal_count as usize,
        );

        Ok(LineSpan {
            start: inserted_line_start as u32,
            count: reveal_count,
        })
    }

    /// Reveals every remaining region; the palette drives this so keyboard-only review can expand context without per-region targeting.
    pub fn expand_all(&mut self) -> Result<ContextExpansionResult, ContextExpansionError> {
        loop {
            let Some(region) = self.diff.lines.iter().find_map(|line| line.context_region) else {
                return Ok(ContextExpansionResult {
                    diff: self.diff.clone(),
                    inserted: LineSpan { start: 0, count: 0 },
                });
            };
            self.expand_in_place(region.id, ContextExpansion::ShowAll)?;
        }
    }

    fn context_lines(
        &self,
        region_id: u32,
        old_start_line: u32,
        new_start_line: u32,
        line_count: u32,
    ) -> Result<Vec<DiffLine>, ContextExpansionError> {
        let mut lines = Vec::with_capacity(line_count as usize);

        for offset in 0..line_count {
            let old_line_no = old_start_line
                .checked_add(offset)
                .ok_or(ContextExpansionError::InvalidRegion { region_id })?;
            let new_line_no = new_start_line
                .checked_add(offset)
                .ok_or(ContextExpansionError::InvalidRegion { region_id })?;
            let Some((_source_offset, text)) =
                self.new_line_index.get(&self.new_content, new_line_no)
            else {
                return Err(ContextExpansionError::MissingSourceLine {
                    line_no: new_line_no,
                });
            };
            lines.push(DiffLine {
                old_line_no: Some(old_line_no),
                new_line_no: Some(new_line_no),
                style: DiffSpanStyle::Context,
                spans: plain_spans(text, DiffSpanStyle::Context),
                conflict_kind: ConflictLineKind::None,
                no_eof_newline: false,
                context_region: None,
            });
        }
        Ok(lines)
    }

    // The first reveal re-renders every visible line from full-source syntax state so constructs opened inside still-hidden regions correct themselves; later reveals only touch the fresh slice, keeping repeated expansion linear.
    fn apply_full_highlights(&mut self, inserted: std::ops::Range<usize>) {
        if should_skip_highlight(&self.diff.path) {
            return;
        }
        let first_pass = self.highlights.is_none();
        let (old_highlights, new_highlights) = self.highlights.get_or_insert_with(|| {
            (
                SideHighlights::full(&self.old_content, &self.old_line_index, &self.diff.language),
                SideHighlights::full(&self.new_content, &self.new_line_index, &self.diff.language),
            )
        });
        if first_pass {
            apply_side_highlights(&mut self.diff.lines, old_highlights, new_highlights);
        } else {
            apply_side_highlights(
                &mut self.diff.lines[inserted],
                old_highlights,
                new_highlights,
            );
        }
    }
}
