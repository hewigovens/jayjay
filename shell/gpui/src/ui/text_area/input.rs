use std::ops::Range;

use gpui::{Bounds, Context, EntityInputHandler, Pixels, Point, UTF16Selection, Window, point};

use super::TextArea;
use crate::ui::input::{TextSelection, sanitize_single_line};

impl TextArea {
    pub(super) fn ensure_focus_handlers(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.focus_subscriptions.is_empty() {
            return;
        }
        let focus_handle = self.focus_handle.clone();
        self.focus_subscriptions = vec![
            cx.on_focus(&focus_handle, window, |input, _window, cx| {
                input.show_caret(cx);
            }),
            cx.on_blur(&focus_handle, window, |input, _window, cx| {
                input.hide_caret(cx);
            }),
        ];
        if self.focus_handle.is_focused(window) {
            self.show_caret(cx);
        }
    }

    pub(super) fn caret_visible(&self) -> bool {
        self.caret.visible()
    }

    pub(super) fn show_caret(&mut self, cx: &mut Context<Self>) {
        self.caret.show(cx, |input, generation, cx| {
            input.toggle_caret(generation, cx)
        });
    }

    fn hide_caret(&mut self, cx: &mut Context<Self>) {
        self.caret.hide(cx);
    }

    fn toggle_caret(&mut self, generation: u64, cx: &mut Context<Self>) -> bool {
        self.caret.toggle_if_current(generation, cx)
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    pub(super) fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range.start)..self.offset_from_utf16(range.end)
    }
}

impl EntityInputHandler for TextArea {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(self.selection.range()),
            reversed: self.selection.is_reversed(),
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _: &mut Window, _: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text = if self.multiline {
            new_text.to_owned()
        } else {
            sanitize_single_line(new_text)
        };
        let range = range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or(self.marked_range.clone())
            .unwrap_or_else(|| self.selection.range_owned());
        self.content =
            (self.content[0..range.start].to_owned() + &text + &self.content[range.end..]).into();
        let end = range.start + text.len();
        self.selection.move_to(end, self.content.len());
        self.marked_range = None;
        self.show_caret(cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or(self.marked_range.clone())
            .unwrap_or_else(|| self.selection.range_owned());
        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        self.marked_range =
            (!new_text.is_empty()).then_some(range.start..range.start + new_text.len());
        let new_selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .map(|new_range| new_range.start + range.start..new_range.end + range.start)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());
        self.selection = TextSelection::from_range(new_selected_range, false, self.content.len());
        self.show_caret(cx);
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        let line = layout
            .lines
            .iter()
            .find(|line| line.range.start <= range.start && range.start <= line.range.end)?;
        let start = range
            .start
            .saturating_sub(line.range.start)
            .min(line.range.len());
        let end = range
            .end
            .saturating_sub(line.range.start)
            .min(line.range.len());
        Some(Bounds::from_corners(
            point(
                bounds.left() + line.shaped.x_for_index(start),
                bounds.top() + line.top,
            ),
            point(
                bounds.left() + line.shaped.x_for_index(end),
                bounds.top() + line.top + layout.line_height,
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.offset_to_utf16(self.index_for_mouse_position(point)))
    }
}
