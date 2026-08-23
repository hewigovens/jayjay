use similar::{Algorithm, TextDiff, TextDiffConfig};

mod change_groups;
mod compute;
mod conflicts;
mod context;
mod expand;
mod highlight;
mod highlights;
mod line_diff;
pub mod placeholders;
mod render_highlights;
mod review_fingerprint;
pub mod side_by_side;
mod stats;
pub mod syntax;
mod types;
mod word_diff;
pub mod wrap;

/// Diff algorithm — histogram matches `jj diff` and reads better on code than Myers.
const DIFF_ALGORITHM: Algorithm = Algorithm::Histogram;

pub(crate) fn text_diff_config() -> TextDiffConfig {
    let mut config = TextDiff::configure();
    config.algorithm(DIFF_ALGORITHM);
    config
}

pub use change_groups::{anchor_side_and_number, change_group_for_anchor, change_groups};
pub use compute::{compute_file_diff, compute_file_diff_full, compute_file_diff_full_plain};
pub use conflicts::{build_diff_display_lines, conflict_display_text};
pub use context::collapse_context_with_mapping;
pub use expand::ExpandableDiff;
pub use highlight::{highlight_file, highlight_file_against_base};
pub use placeholders::{is_git_lfs, is_git_submodule};
pub use review_fingerprint::{
    REVIEW_FINGERPRINT_VERSION, ReviewFileSnapshot, ReviewGroupFingerprint,
    canonical_review_snapshot, display_group_canonical_indices,
};
pub use side_by_side::{RowSide, SideBySideRow, build_side_by_side_rows};
pub use stats::count_changed_lines;
pub use syntax::SyntaxToken;
pub use types::{
    ChangeGroup, CollapsedDiff, ConflictBlock, ConflictBlockSection, ConflictLineKind,
    ContextExpansion, ContextExpansionError, ContextExpansionResult, ContextRegion,
    DiffDisplayItem, DiffLine, DiffSide, DiffSpan, DiffSpanStyle, DisplayLineMapping, FileDiff,
    LineSpan,
};
pub use wrap::{
    DEFAULT_WRAP_COLS, WrappedDiffLine, WrappedSbsRow, WrappedSide, sbs_line_to_row,
    visual_index_for_line, visual_index_for_sbs_row, wrap_cols_for_width, wrap_diff_lines,
    wrap_sbs_rows,
};

#[cfg(test)]
mod tests;
