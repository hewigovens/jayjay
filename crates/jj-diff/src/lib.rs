mod compute;
mod context;
mod highlights;
mod line_diff;
pub mod placeholders;
pub mod side_by_side;
pub mod syntax;
mod types;
mod word_diff;

#[cfg(test)]
mod tests;

pub use compute::{compute_file_diff, compute_file_diff_full};
pub use context::collapse_context_with_mapping;
pub use placeholders::{is_editable_text, is_git_lfs, is_git_submodule};
pub use side_by_side::{SideBySideRow, build_side_by_side_rows};
pub use syntax::{HighlightSpan, SyntaxToken, highlight, language_for_path};
pub use types::{CollapsedDiff, DiffLine, DiffSpan, DiffSpanStyle, DisplayLineMapping, FileDiff};
