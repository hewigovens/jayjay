use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

const PATH_SEGMENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

// Escapes `/` too, so a value like `group/project` stays a single segment.
pub fn encode_path_segment(segment: &str) -> String {
    utf8_percent_encode(segment, PATH_SEGMENT).to_string()
}
