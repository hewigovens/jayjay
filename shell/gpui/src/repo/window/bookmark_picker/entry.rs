use jayjay_core::{BookmarkInfo, bookmark_filter_revset};

use crate::repo::window::picker::PickerRow;

#[derive(Clone)]
pub(super) struct BookmarkPickerEntry {
    pub bookmark: BookmarkInfo,
    pub remote: Option<String>,
}

impl BookmarkPickerEntry {
    pub fn id(&self) -> String {
        let name = &self.bookmark.name;
        match &self.remote {
            Some(remote) => format!("bookmark-picker-remote-row-{}:{name}{remote}", name.len()),
            None => format!("bookmark-picker-row-{name}"),
        }
    }

    pub fn label(&self) -> String {
        match &self.remote {
            Some(remote) => format!("{}@{remote}", self.bookmark.name),
            None => self.bookmark.name.clone(),
        }
    }

    pub fn revset(&self) -> String {
        bookmark_filter_revset(&self.bookmark.name, self.remote.as_deref())
    }
}

impl PickerRow for BookmarkPickerEntry {
    type Action = String;

    fn action(&self) -> Option<String> {
        Some(self.revset())
    }
}
