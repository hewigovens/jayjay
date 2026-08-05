mod context_controls;
mod edit_selection;
mod find_bar;
mod gutter_mouse;
mod header;
mod mouse;
mod note_banner;
mod placeholders;
mod render;
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
pub use rows::{DiffRenderRow, DiffRenderRows, NoteDotKind, row_index_for_line};
pub use state::{DetailMode, DiffViewMode, DiffViewState, FindState, SvgPreviewContent};
pub(crate) use wrap_cache::DiffWrapCache;
