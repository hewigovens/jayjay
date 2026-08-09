use gpui::Entity;
use jayjay_core::diff::FileDiff;
use jayjay_core::{ConflictEditorData, MergeHunkSource};

use crate::ui::text_area::TextArea;

#[derive(Default)]
pub(crate) struct ConflictEditorState {
    pub(crate) active: bool,
    pub(crate) preparing: bool,
    pub(crate) focus_pending: bool,
    pub(crate) show_base: bool,
    pub(crate) show_raw: bool,
    pub(crate) selected_hunk: usize,
    pub(crate) session: u64,
    pub(crate) rev: String,
    pub(crate) path: String,
    pub(crate) data: Option<ConflictEditorData>,
    pub(crate) hunk_diffs: Vec<FileDiff>,
    pub(crate) sources: Option<[Entity<TextArea>; 3]>,
    pub(crate) result: Option<Entity<TextArea>>,
    pub(crate) selected_source: Option<(MergeHunkSource, String)>,
    pub(crate) saving: bool,
}
