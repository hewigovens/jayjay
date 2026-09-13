use jayjay_core::ChangeInfo;
use jayjay_core::compare::{
    self, BookmarkDiffRequest, CompareDisplay, CompareState, RevsetEndpoint,
};

#[uniffi::export]
fn quoted_symbol(symbol: String) -> String {
    compare::quoted_symbol(&symbol)
}

/// `BookmarkInfo` only crosses the boundary in the desktop profile.
#[cfg(feature = "desktop")]
#[uniffi::export]
fn bookmark_endpoint_for_info(bookmark: jayjay_core::BookmarkInfo) -> RevsetEndpoint {
    RevsetEndpoint::for_bookmark(&bookmark)
}

#[uniffi::export]
fn trunk_endpoint() -> RevsetEndpoint {
    RevsetEndpoint::trunk()
}

#[uniffi::export]
fn bookmark_diff_request(base: ChangeInfo, head: ChangeInfo) -> Option<BookmarkDiffRequest> {
    BookmarkDiffRequest::between(&base, &head)
}

#[uniffi::export]
fn bookmark_diff_compare_state(request: BookmarkDiffRequest) -> CompareState {
    request.compare_state()
}

#[uniffi::export]
fn compare_display(from_rev: String, to_rev: String, changes: Vec<ChangeInfo>) -> CompareDisplay {
    CompareDisplay::for_revsets(&from_rev, &to_rev, &changes)
}

#[uniffi::export]
fn combined_compare_state(changes: Vec<ChangeInfo>) -> Option<CompareState> {
    CompareState::combined(&changes)
}

#[uniffi::export]
fn reversed_compare_display(display: CompareDisplay) -> CompareDisplay {
    display.reversed()
}
