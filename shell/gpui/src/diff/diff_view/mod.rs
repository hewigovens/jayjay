mod context_controls;
mod edit_selection;
mod find_bar;
mod gutter_mouse;
mod header;
mod mouse;
mod note_banner;
mod placeholders;
mod render;
mod review_row_map;
mod review_stripe;
mod rows;
mod sbs_body;
mod sbs_note_banner;
mod state;
mod unified_body;
mod wrap_cache;

pub(crate) use edit_selection::{
    display_range_to_diff_edit_range, selection_covers_whole_change_group,
};
pub use render::diff_view;
pub(crate) use rows::row_index_for_line;
pub use rows::{DiffRenderRow, DiffRenderRows, NoteDotKind};
pub use state::{DetailMode, DiffViewMode};
pub(crate) use state::{DiffViewState, FindState, ReviewDisplayState, SvgPreviewContent};
pub(crate) use wrap_cache::DiffWrapCache;
