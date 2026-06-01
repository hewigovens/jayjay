mod caret;
mod line_edit;
mod render;
mod selection;

pub use caret::CaretBlink;
pub use line_edit::{LineEdit, LineEditKeyResult};
pub use render::line_edit_content;
pub(crate) use render::selection_bg;
pub use selection::{
    TextSelection, line_range_at, line_ranges, next_boundary, next_word_boundary,
    previous_boundary, previous_word_boundary, sanitize_single_line,
};
