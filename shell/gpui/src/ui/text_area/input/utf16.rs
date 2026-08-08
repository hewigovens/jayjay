use std::ops::Range;

use super::super::TextArea;

/// UTF-8 byte offset of a UTF-16 code-unit offset within `text`.
///
/// Maps AppKit's marked-text selection against the marked substring (not the whole
/// content), so the result lands on a char boundary of `text`.
pub(super) fn offset_in_str_from_utf16(text: &str, offset: usize) -> usize {
    let mut utf8_offset = 0;
    let mut utf16_count = 0;
    for ch in text.chars() {
        if utf16_count >= offset {
            break;
        }
        utf16_count += ch.len_utf16();
        utf8_offset += ch.len_utf8();
    }
    utf8_offset
}

impl TextArea {
    fn offset_from_utf16(&self, offset: usize) -> usize {
        offset_in_str_from_utf16(&self.content, offset)
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

    pub(super) fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    pub(super) fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range.start)..self.offset_from_utf16(range.end)
    }
}

#[cfg(test)]
mod tests {
    use super::offset_in_str_from_utf16;

    #[test]
    fn maps_utf16_offsets_within_a_substring() {
        // "あ" is 1 UTF-16 unit, 3 UTF-8 bytes. Mapping it against itself must
        // land on the substring's own char boundaries, not those of any prefix.
        assert_eq!(offset_in_str_from_utf16("あ", 0), 0);
        assert_eq!(offset_in_str_from_utf16("あ", 1), 3);
        // Surrogate pair (emoji) is 2 UTF-16 units, 4 UTF-8 bytes.
        assert_eq!(offset_in_str_from_utf16("😀x", 2), 4);
        assert_eq!(offset_in_str_from_utf16("😀x", 3), 5);
    }
}
