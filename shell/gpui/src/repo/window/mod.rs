mod actions;
mod dag;
mod dag_row;
mod detail;
mod diff_select;
mod drag;
mod find;
mod menu;
mod nav;
mod open;
mod render;
mod sidebar;
mod status_bar;
mod view;

pub use open::open_repo_window;
pub use view::{ActivePane, PanelBoundsSlot, RepoWindow};

pub(crate) use view::{
    ColumnDrag, DESCRIPTION_MAX, DESCRIPTION_MIN, DragTarget, FILE_COLUMN_MAX, FILE_COLUMN_MIN,
    SIDEBAR_MAX, SIDEBAR_MIN, TextModalAction, TextModalState,
};
