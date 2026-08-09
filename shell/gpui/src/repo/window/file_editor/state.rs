use gpui::Entity;
use jayjay_core::FileEditorData;

use crate::ui::text_area::TextArea;

#[derive(Default)]
pub(crate) struct FileEditorState {
    pub(crate) active: bool,
    pub(crate) preparing: bool,
    pub(crate) focus_pending: bool,
    pub(crate) session: u64,
    pub(crate) path: String,
    pub(crate) data: Option<FileEditorData>,
    pub(crate) editor: Option<Entity<TextArea>>,
    pub(crate) saving: bool,
}
