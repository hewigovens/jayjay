use jayjay_core::{BookmarkInfo, ChangeInfo};

const TRUNK_BOOKMARKS: &[&str] = &["main", "master", "trunk"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevsetEndpoint {
    pub rev: String,
    pub label: String,
}

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
pub struct PrDiffRequest {
    pub base: RevsetEndpoint,
    pub head: RevsetEndpoint,
    pub head_change_id: String,
}

impl PrDiffRequest {
    pub fn compare_from_rev(&self) -> String {
        pr_diff_base(&self.base.rev, &self.head.rev)
    }

    pub fn compare_state(&self) -> CompareState {
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

pub fn change_revision(change: &ChangeInfo) -> String {
    if change.is_divergent {
        change.commit_id.clone()
    } else {
        change.change_id.clone()
    }
}

pub fn change_label(change: &ChangeInfo) -> String {
    if let Some(bookmark) = change.bookmarks.first()
        && !bookmark.is_empty()
    {
        return bookmark.clone();
    }
    if change.is_working_copy {
        return "@".to_string();
    }
    change.change_id.chars().take(8).collect()
}

pub fn compare_state(from: &ChangeInfo) -> CompareState {
    CompareState {
        from_rev: change_revision(from),
        to_rev: String::new(),
        source_change_id: Some(from.change_id.clone()),
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
    state.target_change_id = Some(to.change_id.clone());
    state.display.to = change_label(to);
    state
}

pub fn is_trunk_bookmark(name: &str) -> bool {
    let bare = name.split('@').next().unwrap_or(name);
    TRUNK_BOOKMARKS.contains(&bare)
}

pub fn bookmark_endpoint(name: &str) -> RevsetEndpoint {
    RevsetEndpoint {
        rev: quoted_symbol(name),
        label: name.to_string(),
    }
}

pub fn bookmark_endpoint_for_info(bookmark: &BookmarkInfo) -> RevsetEndpoint {
    if !bookmark.has_local_target
        && let Some(remote) = bookmark.available_remotes.first()
    {
        return bookmark_endpoint(&format!("{}@{remote}", bookmark.name));
    }
    bookmark_endpoint(&bookmark.name)
}

pub fn trunk_endpoint() -> RevsetEndpoint {
    RevsetEndpoint {
        rev: "trunk()".to_string(),
        label: "trunk".to_string(),
    }
}

pub fn pr_diff_base(base: &str, head: &str) -> String {
    format!("fork_point({base} | {head})")
}

pub fn quoted_symbol(symbol: &str) -> String {
    let escaped = symbol.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
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

pub fn pr_diff_request(base: &ChangeInfo, head: &ChangeInfo) -> Option<PrDiffRequest> {
    let base = primary_base_bookmark_endpoint(base)?;
    let head_endpoint = primary_head_bookmark_endpoint(head)?;
    if base.label == head_endpoint.label {
        return None;
    }
    Some(PrDiffRequest {
        base,
        head: head_endpoint,
        head_change_id: head.change_id.clone(),
    })
}

pub fn trunk_pr_diff_request(head: &ChangeInfo, bookmark: &str) -> Option<PrDiffRequest> {
    if is_trunk_bookmark(bookmark) {
        return None;
    }
    Some(PrDiffRequest {
        base: trunk_endpoint(),
        head: bookmark_endpoint(bookmark),
        head_change_id: head.change_id.clone(),
    })
}

#[cfg(test)]
mod tests {
    use jayjay_core::{ChangeInfo, CommitAuthor};

    use super::*;

    #[test]
    fn quotes_bookmark_symbols() {
        assert_eq!(quoted_symbol("feature-x"), "\"feature-x\"");
        assert_eq!(quoted_symbol("feature\"x"), "\"feature\\\"x\"");
    }

    #[test]
    fn builds_pr_diff_request_from_bookmarked_changes() {
        let base = change("base", &["main"]);
        let head = change("head", &["feature"]);
        let request = pr_diff_request(&base, &head).expect("request");

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
    fn divergent_changes_resolve_by_commit_id() {
        let mut change = change("change-id", &[]);
        change.commit_id = "commit-id".to_string();
        change.is_divergent = true;

        assert_eq!(change_revision(&change), "commit-id");
    }

    #[test]
    fn skips_trunk_head() {
        let base = change("base", &["feature"]);
        let head = change("head", &["main"]);

        assert!(pr_diff_request(&base, &head).is_none());
    }

    fn change(change_id: &str, bookmarks: &[&str]) -> ChangeInfo {
        ChangeInfo {
            change_id: change_id.to_string(),
            commit_id: format!("{change_id}-commit"),
            description: "entry".to_string(),
            author: CommitAuthor::empty(0),
            parents: Vec::new(),
            bookmarks: bookmarks.iter().map(|name| (*name).to_string()).collect(),
            is_working_copy: false,
            has_conflict: false,
            is_empty: false,
            is_immutable: false,
            is_divergent: false,
        }
    }
}
