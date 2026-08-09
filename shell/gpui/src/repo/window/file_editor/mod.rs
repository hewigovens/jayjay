mod actions;
mod state;
mod view;

pub(crate) use state::FileEditorState;
pub(in crate::repo::window) use view::file_editor_overlay;
