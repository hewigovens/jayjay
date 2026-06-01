use gpui::{Context, Window};

use super::super::TextArea;
use super::super::action::{Down, SelectDown, SelectUp, Up};

impl TextArea {
    pub(in crate::ui::text_area) fn up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.offset_for_vertical_move(-1), cx);
    }

    pub(in crate::ui::text_area) fn down(
        &mut self,
        _: &Down,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_to(self.offset_for_vertical_move(1), cx);
    }

    pub(in crate::ui::text_area) fn select_up(
        &mut self,
        _: &SelectUp,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.offset_for_vertical_move(-1), cx);
    }

    pub(in crate::ui::text_area) fn select_down(
        &mut self,
        _: &SelectDown,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.offset_for_vertical_move(1), cx);
    }

    fn offset_for_vertical_move(&self, delta: isize) -> usize {
        self.layout_vertical_offset(delta)
            .unwrap_or_else(|| self.text_vertical_offset(delta))
    }

    fn layout_vertical_offset(&self, delta: isize) -> Option<usize> {
        let layout = self.last_layout.as_ref()?;
        let cursor = self.cursor_offset();
        let current_ix = layout
            .lines
            .iter()
            .position(|line| line.range.start <= cursor && cursor <= line.range.end)
            .unwrap_or_else(|| layout.lines.len().saturating_sub(1));
        let target_ix = current_ix.checked_add_signed(delta)?;
        let target = layout.lines.get(target_ix)?;
        let current = &layout.lines[current_ix];
        let local = cursor
            .saturating_sub(current.range.start)
            .min(current.range.len());
        let x = current.shaped.x_for_index(local);
        let target_local = target.shaped.closest_index_for_x(x).min(target.range.len());
        Some(target.range.start + target_local)
    }

    fn text_vertical_offset(&self, delta: isize) -> usize {
        let ranges = self.line_ranges();
        let cursor = self.cursor_offset();
        let current_ix = ranges
            .iter()
            .position(|range| range.start <= cursor && cursor <= range.end)
            .unwrap_or_else(|| ranges.len().saturating_sub(1));
        let Some(target_ix) = current_ix.checked_add_signed(delta) else {
            return if delta < 0 { 0 } else { self.content.len() };
        };
        let Some(current) = ranges.get(current_ix) else {
            return cursor;
        };
        let Some(target) = ranges.get(target_ix) else {
            return if delta < 0 { 0 } else { self.content.len() };
        };
        let column = self.content[current.start..cursor.min(current.end)]
            .chars()
            .count();
        offset_for_char_column(self.content.as_ref(), target, column)
    }
}

fn offset_for_char_column(text: &str, range: &std::ops::Range<usize>, column: usize) -> usize {
    text[range.clone()]
        .char_indices()
        .nth(column)
        .map(|(ix, _)| range.start + ix)
        .unwrap_or(range.end)
}
