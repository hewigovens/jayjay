use jayjay_core::ChangeInfo;

pub fn change_revision(change: &ChangeInfo) -> String {
    if change.is_divergent {
        change.commit_id.id.clone()
    } else {
        change.change_id.id.clone()
    }
}

pub fn change_label(change: &ChangeInfo) -> String {
    if let Some(name) = change.bookmarks.first().or(change.tags.first())
        && !name.is_empty()
    {
        return name.clone();
    }
    if change.is_working_copy {
        return "@".to_string();
    }
    change.change_id.chars().take(8).collect()
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
    fn labels_prefer_bookmarks_then_tags_over_change_ids() {
        let mut change = change("change-id-long", &[]);
        assert_eq!(change_label(&change), "change-i");
        change.tags.push("v1.0.0".to_string());
        assert_eq!(change_label(&change), "v1.0.0");
        change.bookmarks.push("main".to_string());
        assert_eq!(change_label(&change), "main");
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
