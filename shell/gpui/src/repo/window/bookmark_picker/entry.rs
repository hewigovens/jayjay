use jayjay_core::{BookmarkInfo, DEFAULT_REVSET_DEPTH};

use crate::repo::revset::quoted_symbol;
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
        let name = quoted_symbol(&self.bookmark.name);
        match &self.remote {
            Some(remote) => format!(
                "ancestors(remote_bookmarks(exact:{name}, exact:{}), {DEFAULT_REVSET_DEPTH})",
                quoted_symbol(remote)
            ),
            None if self.bookmark.is_conflicted => format!("bookmarks(exact:{name})"),
            None => name,
        }
    }
}

impl PickerRow for BookmarkPickerEntry {
    type Action = String;

    fn action(&self) -> Option<String> {
        Some(self.revset())
    }
}
