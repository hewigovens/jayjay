mod actions;
mod bookmark_drag;
mod bookmark_menu;
mod conflicts;
mod dag;
mod dag_row;
mod detail;
mod diff_select;
mod drag;
mod find;
mod menu;
mod nav;
mod onboarding;
mod open;
mod render;
mod review;
mod sidebar;
mod status_bar;
mod sync;
mod view;

pub use open::open_repo_window;
pub use review::install_in_memory as install_in_memory_review_store;
pub use view::{ActivePane, PanelBoundsSlot, RepoWindow};

pub(crate) use view::{
    ColumnDrag, DESCRIPTION_DEFAULT, DESCRIPTION_MAX, DESCRIPTION_MIN, DiffRichPreviewKind,
    DiffRichPreviewSelection, DiffWrapCacheSlot, DragTarget, FILE_COLUMN_MAX, FILE_COLUMN_MIN,
    FileTreeCacheSlot, SIDEBAR_MAX, SIDEBAR_MIN, TextModalAction, TextModalState,
};
