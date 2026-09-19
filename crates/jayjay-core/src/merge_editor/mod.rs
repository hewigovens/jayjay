mod hunks;
mod scroll;

pub use hunks::MergeEditorHunkExt;
pub(crate) use hunks::{is_conflict_marker_line, merge_editor_hunks};
pub use scroll::MergeScrollMap;
