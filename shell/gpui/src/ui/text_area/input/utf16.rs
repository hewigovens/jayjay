use std::ops::Range;

use jayjay_core::utf16::{byte_offset, utf16_offset};

use super::super::TextArea;

impl TextArea {
    pub(super) fn offset_to_utf16(&self, offset: usize) -> usize {
        utf16_offset(&self.content, offset)
    }

    pub(super) fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    pub(super) fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        byte_offset(&self.content, range.start)..byte_offset(&self.content, range.end)
    }
}
