use jayjay_core::{BookmarkFilterTarget, BookmarkInfo};

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

    pub fn target(&self) -> Option<BookmarkFilterTarget> {
        let label = self.label();
        BookmarkFilterTarget::for_bookmark(&self.bookmark)
            .into_iter()
            .find(|target| target.name == label)
    }
}

impl PickerRow for BookmarkPickerEntry {
    type Action = BookmarkFilterTarget;

    fn action(&self) -> Option<BookmarkFilterTarget> {
        self.target()
    }
}
