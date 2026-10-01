use crate::ChangeInfo;

#[derive(Debug, Clone)]
pub struct MergeParentChoice {
    pub label: String,
    pub parents: Vec<String>,
}

impl MergeParentChoice {
    pub fn for_selection(changes: &[ChangeInfo]) -> Vec<Self> {
        let revisions: Vec<_> = changes
            .iter()
            .map(|change| change.selection_revision().to_owned())
            .collect();
        changes
            .iter()
            .enumerate()
            .map(|(index, change)| {
                let mut parents = revisions.clone();
                let first = parents.remove(index);
                parents.insert(0, first);
                let id = if change.is_divergent {
                    &change.commit_id
                } else {
                    &change.change_id
                };
                let mut parts = vec![id.prefix(8.max(id.short_len as usize))];
                if let Some(bookmark) = change.bookmarks.first().filter(|name| !name.is_empty()) {
                    parts.push(Self::abbreviate(bookmark, 24));
                }
                let description = change.description.lines().next().unwrap_or("").trim();
                parts.push(if description.is_empty() {
                    "(no description)".to_owned()
                } else {
                    Self::abbreviate(description, 40)
                });
                Self {
                    label: parts.join(" · "),
                    parents,
                }
            })
            .collect()
    }

    fn abbreviate(text: &str, limit: usize) -> String {
        let mut chars = text.chars();
        let mut result: String = chars.by_ref().take(limit).collect();
        if chars.next().is_some() {
            result.push('…');
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::graph_entry;

    #[test]
    fn choosing_any_parent_preserves_the_remaining_order_and_divergent_identity() {
        let mut changes: Vec<_> = ["a", "b", "c"].map(|id| graph_entry(id, &[]).change).into();
        changes[1].is_divergent = true;
        changes[1].change_id.id = changes[0].change_id.id.clone();
        let choices = MergeParentChoice::for_selection(&changes);
        let a = changes[0].selection_revision();
        let b = changes[1].selection_revision();
        let c = changes[2].selection_revision();
        assert_ne!(a, b);
        assert_eq!(choices[0].parents, [a, b, c]);
        assert_eq!(choices[1].parents, [b, a, c]);
        assert_eq!(choices[2].parents, [c, a, b]);
        assert_ne!(choices[0].label, choices[1].label);
    }

    #[test]
    fn labels_keep_identity_with_long_unicode_names_and_empty_descriptions() {
        let mut change = graph_entry("abcdef123456", &[]).change;
        change.bookmarks = vec!["界".repeat(30)];
        change.description = format!("{}\nbody", "文".repeat(50));
        let choices = MergeParentChoice::for_selection(&[change.clone()]);
        assert!(choices[0].label.contains(&format!("{}…", "界".repeat(24))));
        assert!(choices[0].label.ends_with(&format!("{}…", "文".repeat(40))));
        change.description.clear();
        assert!(
            MergeParentChoice::for_selection(&[change])[0]
                .label
                .ends_with("(no description)")
        );
    }
}
