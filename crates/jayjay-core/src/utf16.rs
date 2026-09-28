//! UTF-16 code-unit offsets, the unit AppKit reports for a text field's caret and selection.

/// UTF-8 byte offset of a UTF-16 code-unit offset within `text`.
pub fn byte_offset(text: &str, offset: usize) -> usize {
    let mut byte_offset = 0;
    let mut utf16_count = 0;
    for ch in text.chars() {
        if utf16_count >= offset {
            break;
        }
        utf16_count += ch.len_utf16();
        byte_offset += ch.len_utf8();
    }
    byte_offset
}

/// UTF-16 code-unit offset of a UTF-8 byte offset within `text`.
pub fn utf16_offset(text: &str, offset: usize) -> usize {
    let mut utf16_offset = 0;
    let mut byte_count = 0;
    for ch in text.chars() {
        if byte_count >= offset {
            break;
        }
        byte_count += ch.len_utf8();
        utf16_offset += ch.len_utf16();
    }
    utf16_offset
}

#[cfg(test)]
mod tests {
    use super::{byte_offset, utf16_offset};

    #[test]
    fn maps_offsets_between_utf16_units_and_utf8_bytes() {
        // "あ" is 1 UTF-16 unit, 3 UTF-8 bytes; an emoji is a surrogate pair.
        assert_eq!(byte_offset("あ", 0), 0);
        assert_eq!(byte_offset("あ", 1), 3);
        assert_eq!(byte_offset("😀x", 2), 4);
        assert_eq!(byte_offset("😀x", 3), 5);
        assert_eq!(utf16_offset("あ", 0), 0);
        assert_eq!(utf16_offset("あ", 3), 1);
        assert_eq!(utf16_offset("😀x", 4), 2);
        assert_eq!(utf16_offset("😀x", 5), 3);

        for text in ["", "ascii", "あ😀x", "mixed ünïcode"] {
            let (mut bytes, mut utf16) = (0, 0);
            loop {
                assert_eq!(byte_offset(text, utf16), bytes, "{text} at {utf16}");
                assert_eq!(utf16_offset(text, bytes), utf16, "{text} at {bytes}");
                match text[bytes..].chars().next() {
                    Some(ch) => {
                        bytes += ch.len_utf8();
                        utf16 += ch.len_utf16();
                    }
                    None => break,
                }
            }
        }
    }
}
