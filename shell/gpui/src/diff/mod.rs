mod annotate_view;
mod bounds;
mod diff_view;
mod file_column;
pub(crate) mod file_status;
mod image_diff;
mod image_diff_split;
pub(crate) mod line;
mod line_stats;
mod markdown_diff;
pub(crate) use markdown_diff::{MarkdownImageCache, MarkdownImageCacheSlot};
mod media_diff;
pub(crate) mod projection;
mod read_only;
mod selection;
mod side_by_side;
mod spans;
mod svg_diff;
mod svg_preview;
pub(crate) mod wrap;

pub(crate) use annotate_view::Annotation;
pub(crate) use bounds::{bounds_capture, column_bounds_capture};
pub(crate) use diff_view::{
    DETAIL_INSET, DiffShape, DiffViewState, DiffWrapCache, FindState, ReviewDisplayState,
    SvgPreviewContent, diff_view, display_range_to_diff_edit_range, edit_diff_button, mode_label,
    row_index_for_line, selection_covers_whole_change_group, show_all_files_button,
    stale_notes_banner, view_mode_button,
};
pub use diff_view::{DetailMode, DiffRenderRow, DiffRenderRows, DiffViewMode, NoteDotKind};
pub(crate) use diff_view::{
    StackedFileHeader, StackedRows, placeholder_inner, render_find_bar, stacked_file_header,
};
pub(crate) use file_column::{FileColumnState, file_column};
pub(crate) use file_column::{FileTreeCache, file_row_height, middle_elide};
pub(crate) use image_diff::{hunk_is_image, image_diff_view};
pub(crate) use line_stats::line_stats;
pub(crate) use read_only::read_only_diff;
pub(crate) use selection::word_at;
pub use selection::{DiffSelection, GutterLineSelection, SbsSide};
