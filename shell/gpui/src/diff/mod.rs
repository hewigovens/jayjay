mod annotate_view;
mod bounds;
mod diff_view;
mod file_column;
pub(crate) mod file_status;
mod image_diff;
pub(crate) mod line;
mod markdown_diff;
mod media_diff;
pub(crate) mod projection;
mod selection;
mod side_by_side;
mod spans;
mod svg_diff;
pub(crate) mod wrap;

pub(crate) use bounds::bounds_capture;
pub use diff_view::{DetailMode, DiffRenderRow, DiffRenderRows, DiffViewMode, NoteDotKind};
pub(crate) use diff_view::{
    DiffViewState, DiffWrapCache, FindState, ReviewDisplayState, SvgPreviewContent, diff_view,
    display_range_to_diff_edit_range, row_index_for_line, selection_covers_whole_change_group,
};
pub(crate) use file_column::{FileColumnState, file_column};
pub(crate) use file_column::{FileTreeCache, middle_elide};
pub(crate) use image_diff::{hunk_is_image, image_diff_view};
pub(crate) use selection::word_at;
pub use selection::{DiffSelection, GutterLineSelection, SbsSide};
