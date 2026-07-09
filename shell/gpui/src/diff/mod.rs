mod annotate_view;
mod diff_view;
mod file_column;
mod file_status;
mod image_diff;
mod line;
mod markdown_diff;
mod media_diff;
pub(crate) mod projection;
mod selection;
mod side_by_side;
mod spans;
mod svg_diff;
pub(crate) mod wrap;

pub use diff_view::{
    DetailMode, DiffRenderRow, DiffRenderRows, DiffViewMode, DiffViewState, FindState, NoteDotKind,
    SvgPreviewContent, diff_view, row_index_for_line,
};
pub(crate) use diff_view::{
    DiffWrapCache, display_range_to_diff_edit_range, selection_covers_whole_change_group,
};
pub use file_column::{FileColumnState, file_column};
pub(crate) use file_column::{FileTreeCache, middle_elide};
pub use selection::{DiffSelection, GutterLineSelection, SbsSide, word_at};
