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
    pub fn at(offset: usize) -> Self {
        Self {
            range: offset..offset,
            reversed: false,
        }
    }

    pub fn from_range(range: Range<usize>, reversed: bool, text_len: usize) -> Self {
        let start = range.start.min(text_len);
        let end = range.end.min(text_len);
        let (range, reversed) = if end < start {
            (end..start, !reversed)
        } else {
            (start..end, reversed)
        };
        Self { range, reversed }
    }

    pub fn range(&self) -> &Range<usize> {
        &self.range
    }

    pub fn range_owned(&self) -> Range<usize> {
        self.range.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.range.is_empty()
    }

    pub fn is_reversed(&self) -> bool {
        self.reversed
    }

    pub fn cursor_offset(&self) -> usize {
        if self.reversed {
            self.range.start
        } else {
            self.range.end
        }
    }

    pub fn move_to(&mut self, offset: usize, text_len: usize) {
        let offset = offset.min(text_len);
        self.range = offset..offset;
        self.reversed = false;
    }

    pub fn select_to(&mut self, offset: usize, text_len: usize) {
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

    pub fn select_all(&mut self, text_len: usize) {
        self.range = 0..text_len;
        self.reversed = false;
    }
}
