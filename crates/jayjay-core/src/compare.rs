mod endpoint;
mod request;
mod state;

pub use endpoint::{RevsetEndpoint, quoted_symbol};
pub use request::BookmarkDiffRequest;
pub use state::{CompareDisplay, CompareState, combined_diff_revsets};

#[cfg(test)]
mod tests {
    use crate::{ChangeInfo, CommitAuthor, NewChangeEligibility, ShortId};

    pub(super) fn change(change_id: &str, bookmarks: &[&str]) -> ChangeInfo {
        ChangeInfo {
            change_id: ShortId::new(change_id.to_owned(), 1),
            commit_id: ShortId::new(format!("{change_id}-commit"), 1),
            description: "entry".to_owned(),
            author: CommitAuthor::empty(0),
            parents: Vec::new(),
            bookmarks: bookmarks.iter().map(|name| (*name).to_owned()).collect(),
            tags: Vec::new(),
            workspaces: Vec::new(),
            is_working_copy: false,
            has_conflict: false,
            is_empty: false,
            is_immutable: false,
            is_divergent: false,
            new_change: NewChangeEligibility {
                on_top: true,
                before: true,
                after: true,
            },
        }
    }
}
