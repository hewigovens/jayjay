use std::ops::Range;

#[derive(Debug, Clone)]
pub struct TextSelection {
    range: Range<usize>,
    reversed: bool,
}

impl Default for TextSelection {
    fn default() -> Self {
        Self::at(0)
    }
}

impl TextSelection {
    pub(crate) fn at(offset: usize) -> Self {
        Self {
            range: offset..offset,
            reversed: false,
        }
    }

    pub(crate) fn from_range(range: Range<usize>, reversed: bool, text_len: usize) -> Self {
        let start = range.start.min(text_len);
        let end = range.end.min(text_len);
        let (range, reversed) = if end < start {
            (end..start, !reversed)
        } else {
            (start..end, reversed)
        };
        Self { range, reversed }
    }

    pub(crate) fn range(&self) -> &Range<usize> {
        &self.range
    }

    pub(crate) fn range_owned(&self) -> Range<usize> {
        self.range.clone()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.range.is_empty()
    }

    pub(crate) fn is_reversed(&self) -> bool {
        self.reversed
    }

    pub(crate) fn cursor_offset(&self) -> usize {
        if self.reversed {
            self.range.start
        } else {
            self.range.end
        }
    }

    pub(crate) fn move_to(&mut self, offset: usize, text_len: usize) {
        let offset = offset.min(text_len);
        self.range = offset..offset;
        self.reversed = false;
    }

    pub(crate) fn select_to(&mut self, offset: usize, text_len: usize) {
        let offset = offset.min(text_len);
        if self.reversed {
            self.range.start = offset;
        } else {
            self.range.end = offset;
        }
        if self.range.end < self.range.start {
            self.reversed = !self.reversed;
            self.range = self.range.end..self.range.start;
        }
    }

    pub(crate) fn select_all(&mut self, text_len: usize) {
        self.range = 0..text_len;
        self.reversed = false;
    }
}
