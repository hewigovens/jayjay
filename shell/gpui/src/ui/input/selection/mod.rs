mod boundary;
mod state;

pub use boundary::{
    line_range_at, line_ranges, next_boundary, next_word_boundary, previous_boundary,
    previous_word_boundary, sanitize_single_line,
};
pub use state::TextSelection;
