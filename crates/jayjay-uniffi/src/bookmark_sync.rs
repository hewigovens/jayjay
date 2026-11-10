use jayjay_core::{BookmarkInfo, BookmarkSync};

#[uniffi::remote(Record)]
pub struct BookmarkSync {
    pub text: String,
    pub tooltip: String,
}

#[uniffi::export]
fn bookmark_sync(bookmark: BookmarkInfo) -> Option<BookmarkSync> {
    BookmarkSync::for_bookmark(&bookmark)
}
