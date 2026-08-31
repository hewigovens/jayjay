use jayjay_core::ChangeInfo;

const TRUNK_BOOKMARKS: &[&str] = &["main", "master", "trunk"];

pub fn change_revision(change: &ChangeInfo) -> String {
    if change.is_divergent {
        change.commit_id.id.clone()
    } else {
        change.change_id.id.clone()
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

pub fn is_trunk_bookmark(name: &str) -> bool {
    let bare = name.split('@').next().unwrap_or(name);
    TRUNK_BOOKMARKS.contains(&bare)
}

/// DAG chips may drop a conflicted target even on trunk; whole-bookmark delete stays hidden for resolved trunk names.
pub fn can_remove_bookmark_from_chip(name: &str, conflicted: bool) -> bool {
    conflicted || !is_trunk_bookmark(name)
}

#[cfg(test)]
mod tests {
    use jayjay_core::CommitAuthor;

    use super::*;

    #[test]
    fn divergent_changes_resolve_by_commit_id() {
        let mut change = change("change-id", &[]);
        change.commit_id.id = "commit-id".to_string();
        change.is_divergent = true;

        assert_eq!(change_revision(&change), "commit-id");
    }

    #[test]
    fn non_divergent_changes_resolve_by_change_id() {
        let change = change("change-id", &[]);

        assert_eq!(change_revision(&change), "change-id");
    }

    #[test]
    fn chip_can_remove_a_conflicted_trunk_target() {
        assert!(can_remove_bookmark_from_chip("main", true));
        assert!(can_remove_bookmark_from_chip("main@origin", true));
        assert!(can_remove_bookmark_from_chip("feature", false));
        assert!(!can_remove_bookmark_from_chip("main", false));
        assert!(!can_remove_bookmark_from_chip("master", false));
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
