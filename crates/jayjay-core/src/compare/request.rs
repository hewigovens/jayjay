use super::endpoint::RevsetEndpoint;
use super::state::{CompareDisplay, CompareState};
use crate::{ChangeInfo, trunk};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookmarkDiffRequest {
    pub base: RevsetEndpoint,
    pub head: RevsetEndpoint,
    pub head_change_id: String,
}

impl BookmarkDiffRequest {
    pub fn between(base: &ChangeInfo, head: &ChangeInfo) -> Option<Self> {
        let base = primary_base_bookmark_endpoint(base)?;
        let head_endpoint = primary_head_bookmark_endpoint(head)?;
        if base.label == head_endpoint.label {
            return None;
        }
        Some(Self {
            base,
            head: head_endpoint,
            head_change_id: head.change_id.id.clone(),
        })
    }

    pub fn from_trunk(head: &ChangeInfo, bookmark: &str) -> Option<Self> {
        if trunk::is_trunk_bookmark(bookmark) {
            return None;
        }
        Some(Self {
            base: RevsetEndpoint::trunk(),
            head: RevsetEndpoint::bookmark(bookmark),
            head_change_id: head.change_id.id.clone(),
        })
    }

    /// A PR diff starts where the two bookmarks last agreed, not at whatever the base has grown since.
    pub fn compare_state(&self) -> CompareState {
        CompareState {
            from_rev: format!("fork_point({} | {})", self.base.rev, self.head.rev),
            to_rev: self.head.rev.clone(),
            source_change_id: None,
            target_change_id: Some(self.head_change_id.clone()),
            display: CompareDisplay {
                title: "PR Diff".to_owned(),
                from: self.base.label.clone(),
                to: self.head.label.clone(),
                is_combined_selection: false,
            },
        }
    }
}

fn primary_base_bookmark_endpoint(change: &ChangeInfo) -> Option<RevsetEndpoint> {
    change
        .bookmarks
        .iter()
        .find(|name| trunk::is_trunk_bookmark(name))
        .or_else(|| change.bookmarks.first())
        .map(|name| RevsetEndpoint::bookmark(name))
}

fn primary_head_bookmark_endpoint(change: &ChangeInfo) -> Option<RevsetEndpoint> {
    change
        .bookmarks
        .iter()
        .find(|name| !trunk::is_trunk_bookmark(name))
        .map(|name| RevsetEndpoint::bookmark(name))
}

#[cfg(test)]
mod tests {
    use super::super::tests::change;
    use super::*;

    #[test]
    fn a_bookmarked_pair_diffs_from_their_fork_point() {
        let request =
            BookmarkDiffRequest::between(&change("base", &["main"]), &change("head", &["feature"]))
                .expect("request");
        let state = request.compare_state();

        assert_eq!(state.from_rev, "fork_point(\"main\" | \"feature\")");
        assert_eq!(state.to_rev, "\"feature\"");
        assert_eq!(state.target_change_id.as_deref(), Some("head"));
        assert_eq!(state.display.title, "PR Diff");
        assert_eq!(state.display.from, "main");
        assert_eq!(state.display.to, "feature");
    }

    #[test]
    fn a_trunk_head_has_no_pr_diff() {
        assert!(
            BookmarkDiffRequest::between(&change("base", &["feature"]), &change("head", &["main"]))
                .is_none()
        );
        assert!(BookmarkDiffRequest::from_trunk(&change("head", &[]), "main").is_none());
    }

    #[test]
    fn a_bookmark_diff_against_trunk_uses_the_trunk_revset() {
        let request = BookmarkDiffRequest::from_trunk(&change("head", &["feature"]), "feature")
            .expect("request");

        assert_eq!(request.base.rev, "trunk()");
        assert_eq!(
            request.compare_state().from_rev,
            "fork_point(trunk() | \"feature\")"
        );
    }
}
