mod context_controls;
mod edit_selection;
mod find_bar;
mod gutter_mouse;
mod header;
mod mouse;
mod moved_label;
mod note_banner;
mod placeholders;
mod render;
mod review_row_map;
mod review_stripe;
mod row_anchor;
mod row_list;
mod rows;
mod sbs_body;
mod sbs_note_banner;
mod stacked;
mod state;
mod unified_body;
mod wrap_cache;

pub(crate) use edit_selection::{
    display_range_to_diff_edit_range, selection_covers_whole_change_group,
};
pub(crate) use find_bar::render_find_bar;
pub(crate) use header::{
    DETAIL_INSET, collapse_chevron, edit_diff_button, file_collapse_buttons, file_icon,
    file_path_label, hunk_status_pill, mode_label, path_copy_button, source_path_label,
    view_mode_button,
};
pub(crate) use note_banner::stale_notes_banner;
pub(crate) use placeholders::placeholder_row;
pub use render::diff_view;
pub(crate) use rows::row_index_for_line;
pub use rows::{DiffRenderRow, DiffRenderRows, NoteDotKind};
pub(crate) use stacked::{StackedFileHeader, StackedRows, stacked_file_header};
pub use state::{DetailMode, DiffViewMode};
pub(crate) use state::{
    DiffShape, DiffViewState, FindState, ReviewDisplayState, SvgPreviewContent,
};
pub(crate) use wrap_cache::DiffWrapCache;
