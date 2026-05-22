use similar::{Algorithm, TextDiff, TextDiffConfig};

mod compute;
mod context;
mod highlights;
mod line_diff;
pub mod placeholders;
pub mod side_by_side;
pub mod syntax;
mod types;
mod word_diff;
pub mod wrap;

/// Diff algorithm — histogram matches `jj diff` and reads better on code than Myers.
pub(crate) const DIFF_ALGORITHM: Algorithm = Algorithm::Histogram;

/// Pre-configured `TextDiff` builder using [`DIFF_ALGORITHM`].
pub(crate) fn text_diff_config() -> TextDiffConfig {
    let mut config = TextDiff::configure();
    config.algorithm(DIFF_ALGORITHM);
    config
}

#[cfg(test)]
mod tests;

pub use compute::{compute_file_diff, compute_file_diff_full};
pub use context::collapse_context_with_mapping;
pub use placeholders::{is_editable_text, is_git_lfs, is_git_submodule};
pub use side_by_side::{SideBySideRow, build_side_by_side_rows};
pub use syntax::{HighlightSpan, SyntaxToken, highlight, language_for_path};
pub use types::{CollapsedDiff, DiffLine, DiffSpan, DiffSpanStyle, DisplayLineMapping, FileDiff};
pub use wrap::{
    DEFAULT_WRAP_COLS, MIN_WRAP_COLS, WrappedDiffLine, WrappedSbsRow, sbs_line_to_row,
    visual_index_for_line, visual_index_for_sbs_row, wrap_cols_for_width, wrap_diff_lines,
    wrap_sbs_rows,
};
