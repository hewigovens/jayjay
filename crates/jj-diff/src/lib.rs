mod compute;
mod context;
mod highlights;
mod line_diff;
pub mod syntax;
mod types;
mod word_diff;

#[cfg(test)]
mod tests;

pub use compute::{compute_file_diff, compute_file_diff_full};
pub use context::collapse_context_with_mapping;
pub use syntax::{HighlightSpan, SyntaxToken, highlight, language_for_path};
pub use types::{CollapsedDiff, DiffLine, DiffSpan, DiffSpanStyle, DisplayLineMapping, FileDiff};
