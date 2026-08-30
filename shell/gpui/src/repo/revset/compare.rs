use jayjay_core::ChangeInfo;

use super::{
    RevsetEndpoint, bookmark_endpoint, change_label, change_revision, is_trunk_bookmark,
    trunk_endpoint,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompareDisplay {
    pub title: String,
    pub from: String,
    pub to: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompareState {
    pub from_rev: String,
    pub to_rev: String,
    pub source_change_id: Option<String>,
    pub target_change_id: Option<String>,
    pub display: CompareDisplay,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookmarkDiffRequest {
    pub(crate) base: RevsetEndpoint,
    pub(crate) head: RevsetEndpoint,
    pub(crate) head_change_id: String,
}

impl BookmarkDiffRequest {
    fn compare_from_rev(&self) -> String {
        bookmark_diff_base(&self.base.rev, &self.head.rev)
    }

    pub(crate) fn compare_state(&self) -> CompareState {
        CompareState {
            from_rev: self.compare_from_rev(),
            to_rev: self.head.rev.clone(),
            source_change_id: None,
            target_change_id: Some(self.head_change_id.clone()),
            display: CompareDisplay {
                title: "Comparing".to_string(),
                from: self.base.label.clone(),
                to: self.head.label.clone(),
            },
        }
    }
}

pub fn compare_state(from: &ChangeInfo) -> CompareState {
    CompareState {
        from_rev: change_revision(from),
        to_rev: String::new(),
        source_change_id: Some(from.change_id.id.clone()),
        target_change_id: None,
        display: CompareDisplay {
            title: "Comparing".to_string(),
            from: change_label(from),
            to: String::new(),
        },
    }
}

pub fn compare_state_between(from: &ChangeInfo, to: &ChangeInfo) -> CompareState {
    let mut state = compare_state(from);
    state.to_rev = change_revision(to);
    state.target_change_id = Some(to.change_id.id.clone());
    state.display.to = change_label(to);
    state
}

pub fn bookmark_diff_base(base: &str, head: &str) -> String {
    format!("fork_point({base} | {head})")
}

pub fn primary_base_bookmark_endpoint(change: &ChangeInfo) -> Option<RevsetEndpoint> {
    change
        .bookmarks
        .iter()
        .find(|name| is_trunk_bookmark(name))
        .or_else(|| change.bookmarks.first())
        .map(|name| bookmark_endpoint(name))
}

pub fn primary_head_bookmark_endpoint(change: &ChangeInfo) -> Option<RevsetEndpoint> {
    change
        .bookmarks
        .iter()
        .find(|name| !is_trunk_bookmark(name))
        .map(|name| bookmark_endpoint(name))
}

pub fn bookmark_diff_request(base: &ChangeInfo, head: &ChangeInfo) -> Option<BookmarkDiffRequest> {
    let base = primary_base_bookmark_endpoint(base)?;
    let head_endpoint = primary_head_bookmark_endpoint(head)?;
    if base.label == head_endpoint.label {
        return None;
    }
    Some(BookmarkDiffRequest {
        base,
        head: head_endpoint,
        head_change_id: head.change_id.id.clone(),
    })
}

pub fn trunk_bookmark_diff_request(
    head: &ChangeInfo,
    bookmark: &str,
) -> Option<BookmarkDiffRequest> {
    if is_trunk_bookmark(bookmark) {
        return None;
    }
    Some(BookmarkDiffRequest {
        base: trunk_endpoint(),
        head: bookmark_endpoint(bookmark),
        head_change_id: head.change_id.id.clone(),
    })
}

#[cfg(test)]
mod tests {
    use jayjay_core::{ChangeInfo, CommitAuthor};

    use super::*;

    #[test]
    fn builds_bookmark_diff_request_from_bookmarked_changes() {
        let base = change("base", &["main"]);
        let head = change("head", &["feature"]);
        let request = bookmark_diff_request(&base, &head).expect("request");

        assert_eq!(
            request.compare_from_rev(),
            "fork_point(\"main\" | \"feature\")"
        );
        assert_eq!(request.compare_state().display.title, "Comparing");
        assert_eq!(request.compare_state().display.from, "main");
        assert_eq!(request.compare_state().display.to, "feature");
    }

    #[test]
    fn compare_state_prefers_bookmarks() {
        let base = change("base", &["main"]);
        let head = change("head", &["bookmark-diff"]);

        let state = compare_state_between(&head, &base);

        assert_eq!(state.from_rev, "head");
        assert_eq!(state.to_rev, "base");
        assert_eq!(state.source_change_id.as_deref(), Some("head"));
        assert_eq!(state.target_change_id.as_deref(), Some("base"));
        assert_eq!(state.display.title, "Comparing");
        assert_eq!(state.display.from, "bookmark-diff");
        assert_eq!(state.display.to, "main");
    }

    #[test]
    fn skips_trunk_head() {
        let base = change("base", &["feature"]);
        let head = change("head", &["main"]);

        assert!(bookmark_diff_request(&base, &head).is_none());
    }

    fn change(change_id: &str, bookmarks: &[&str]) -> ChangeInfo {
        ChangeInfo {
            change_id: jayjay_core::ShortId::new(change_id.to_string(), 1),
            commit_id: jayjay_core::ShortId::new(format!("{change_id}-commit"), 1),
            description: "entry".to_string(),
            author: CommitAuthor::empty(0),
            parents: Vec::new(),
            bookmarks: bookmarks.iter().map(|name| (*name).to_string()).collect(),
            tags: Vec::new(),
            workspaces: Vec::new(),
            is_working_copy: false,
            has_conflict: false,
            is_empty: false,
            is_immutable: false,
            is_divergent: false,
            new_change: jayjay_core::NewChangeEligibility {
                on_top: true,
                before: true,
                after: true,
            },
        }
    }
}
