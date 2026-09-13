mod endpoint;
mod request;
mod state;

pub use endpoint::{RevsetEndpoint, quoted_symbol};
pub use request::BookmarkDiffRequest;
pub use state::{CompareDisplay, CompareState, combined_diff_revsets};

#[cfg(test)]
mod tests {
    use crate::ChangeInfo;
    use crate::mock::{change_info, strings};

    pub(super) fn change(change_id: &str, bookmarks: &[&str]) -> ChangeInfo {
        let mut change = change_info(change_id, &format!("{change_id}-commit"));
        change.description = "entry".to_owned();
        change.bookmarks = strings(bookmarks);
        change
    }
}
