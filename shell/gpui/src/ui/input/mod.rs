mod caret;
mod edit_history;
mod line_edit;
mod line_input;
mod render;
mod selection;

pub use caret::CaretBlink;
pub(crate) use edit_history::{EditHistory, HistoryStep, TextSnapshot};
pub use line_edit::LineEdit;
pub(crate) use line_edit::LineEditKeyResult;
pub use line_input::LineInput;
pub(crate) use render::{line_input_content, selection_bg};
pub use selection::TextSelection;
pub(crate) use selection::{
    line_range_at, line_ranges, next_boundary, next_word_boundary, previous_boundary,
    previous_word_boundary, sanitize_single_line,
};
