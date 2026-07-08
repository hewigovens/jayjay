mod annotate_view;
mod diff_view;
mod file_column;
mod file_status;
mod image_diff;
mod line;
mod media_diff;
pub(crate) mod projection;
mod selection;
mod side_by_side;
mod spans;
mod svg_diff;
pub(crate) mod wrap;

pub(crate) use diff_view::DiffWrapCache;
pub use diff_view::{
    DetailMode, DiffViewMode, DiffViewState, FindState, SvgPreviewContent, diff_view,
};
pub use file_column::{FileColumnState, file_column};
pub(crate) use file_column::{FileTreeCache, middle_elide};
pub use selection::{DiffSelection, SbsSide, word_at};
