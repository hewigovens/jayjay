use std::ops::Range;

use gpui::{Context, Pixels, Point, px};

use super::super::TextArea;
use crate::ui::input::{
    line_range_at, line_ranges, next_boundary, next_word_boundary, previous_boundary,
    previous_word_boundary,
};

impl TextArea {
    pub(in crate::ui::text_area) fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selection.move_to(offset, self.content.len());
        self.show_caret(cx);
    }

    pub(in crate::ui::text_area) fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selection.select_to(offset, self.content.len());
        self.show_caret(cx);
    }

    pub(in crate::ui::text_area) fn cursor_offset(&self) -> usize {
        self.selection.cursor_offset()
    }

    pub(in crate::ui::text_area) fn index_for_mouse_position(
        &self,
        position: Point<Pixels>,
    ) -> usize {
        if self.content.is_empty() {
            return 0;
        }
        let (Some(bounds), Some(layout)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        let y = (position.y - bounds.top() + self.scroll_y).max(px(0.));
        let line_ix = (f32::from(y) / f32::from(layout.line_height)).floor() as usize;
        let Some(line) = layout.lines.get(line_ix).or_else(|| layout.lines.last()) else {
            return self.content.len();
        };
        let x = position.x - bounds.left();
        let local = line.shaped.closest_index_for_x(x);
        (line.range.start + local).min(line.range.end)
    }

    pub(in crate::ui::text_area) fn previous_boundary(&self, offset: usize) -> usize {
        previous_boundary(self.content.as_ref(), offset)
    }

    pub(in crate::ui::text_area) fn next_boundary(&self, offset: usize) -> usize {
        next_boundary(self.content.as_ref(), offset)
    }

    pub(in crate::ui::text_area) fn previous_word_boundary(&self, offset: usize) -> usize {
        previous_word_boundary(self.content.as_ref(), offset)
    }

    pub(in crate::ui::text_area) fn next_word_boundary(&self, offset: usize) -> usize {
        next_word_boundary(self.content.as_ref(), offset)
    }

    pub(in crate::ui::text_area) fn line_ranges(&self) -> Vec<Range<usize>> {
        line_ranges(self.content.as_ref())
    }

    pub(in crate::ui::text_area) fn line_range_at(&self, offset: usize) -> Range<usize> {
        line_range_at(self.content.as_ref(), offset)
    }
}
