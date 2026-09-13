use crate::{ChangeInfo, ShortId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompareDisplay {
    pub title: String,
    pub from: String,
    pub to: String,
    pub is_combined_selection: bool,
}

impl CompareDisplay {
    pub fn for_revsets(from_rev: &str, to_rev: &str, changes: &[ChangeInfo]) -> Self {
        Self {
            title: "Comparing".to_owned(),
            from: revision_label(from_rev, changes),
            to: revision_label(to_rev, changes),
            is_combined_selection: false,
        }
    }

    pub fn reversed(&self) -> Self {
        Self {
            title: self.title.clone(),
            from: self.to.clone(),
            to: self.from.clone(),
            is_combined_selection: self.is_combined_selection,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompareState {
    pub from_rev: String,
    pub to_rev: String,
    pub source_change_id: Option<String>,
    pub target_change_id: Option<String>,
    pub display: CompareDisplay,
}

impl CompareState {
    pub fn new(from: &ChangeInfo) -> Self {
        Self {
            from_rev: from.selection_revision().to_owned(),
            to_rev: String::new(),
            source_change_id: Some(from.change_id.id.clone()),
            target_change_id: None,
            display: CompareDisplay {
                title: "Comparing".to_owned(),
                from: from.label(),
                to: String::new(),
                is_combined_selection: false,
            },
        }
    }

    pub fn between(from: &ChangeInfo, to: &ChangeInfo) -> Self {
        let mut state = Self::new(from);
        state.to_rev = to.selection_revision().to_owned();
        state.target_change_id = Some(to.change_id.id.clone());
        state.display.to = to.label();
        state
    }

    pub fn combined(changes: &[ChangeInfo]) -> Option<Self> {
        let newest = changes.first()?;
        let oldest = changes.last()?;
        let revisions: Vec<_> = changes
            .iter()
            .map(|change| change.commit_id.id.clone())
            .collect();
        let (from_rev, to_rev) = combined_diff_revsets(&revisions)?;
        Some(Self {
            from_rev,
            to_rev,
            source_change_id: None,
            target_change_id: Some(newest.commit_id.id.clone()),
            display: CompareDisplay {
                title: format!("{} Changes Selected", changes.len()),
                from: oldest.label(),
                to: newest.label(),
                is_combined_selection: true,
            },
        })
    }

    /// Swap both ends. The change ids travel with the revsets they name, so an end without one stays without one.
    pub fn reversed(&self) -> Self {
        Self {
            from_rev: self.to_rev.clone(),
            to_rev: self.from_rev.clone(),
            source_change_id: self.target_change_id.clone(),
            target_change_id: self.source_change_id.clone(),
            display: self.display.reversed(),
        }
    }
}

pub fn combined_diff_revsets(revisions: &[String]) -> Option<(String, String)> {
    let mut unique_revisions = Vec::with_capacity(revisions.len());
    for revision in revisions.iter().map(|revision| revision.trim()) {
        if !revision.is_empty() && !unique_revisions.contains(&revision) {
            unique_revisions.push(revision);
        }
    }
    if unique_revisions.len() < 2 {
        return None;
    }

    let selection = unique_revisions
        .into_iter()
        .map(|revision| format!("({revision})"))
        .collect::<Vec<_>>()
        .join(" | ");
    Some((
        format!("roots({selection})-"),
        format!("heads({selection})"),
    ))
}

fn revision_label(rev: &str, changes: &[ChangeInfo]) -> String {
    if let Some(change) = changes
        .iter()
        .find(|change| change.change_id.id == rev || change.commit_id.id == rev)
    {
        return change.label();
    }
    if let Some(symbol) = rev
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        return symbol.to_owned();
    }
    if rev.contains('(') || rev.contains(' ') {
        return rev.to_owned();
    }
    rev.chars().take(ShortId::LABEL_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::super::tests::change;
    use super::*;

    #[test]
    fn comparing_two_changes_labels_each_end_and_names_both_changes() {
        let state = CompareState::between(
            &change("head", &["bookmark-diff"]),
            &change("base", &["main"]),
        );

        assert_eq!(state.from_rev, "head");
        assert_eq!(state.to_rev, "base");
        assert_eq!(state.source_change_id.as_deref(), Some("head"));
        assert_eq!(state.target_change_id.as_deref(), Some("base"));
        assert_eq!(state.display.title, "Comparing");
        assert_eq!(state.display.from, "bookmark-diff");
        assert_eq!(state.display.to, "main");
    }

    #[test]
    fn reversing_swaps_the_ends_with_the_change_ids_that_name_them() {
        let reversed = CompareState::new(&change("head", &["feature"])).reversed();

        assert_eq!(reversed.from_rev, "");
        assert_eq!(reversed.to_rev, "head");
        assert_eq!(reversed.source_change_id, None);
        assert_eq!(reversed.target_change_id.as_deref(), Some("head"));
        assert_eq!(reversed.display.from, "");
        assert_eq!(reversed.display.to, "feature");
    }

    #[test]
    fn a_combined_selection_diffs_from_the_roots_parent_to_the_heads() {
        let state = CompareState::combined(&[change("newest", &[]), change("oldest", &[])])
            .expect("combined comparison");

        assert_eq!(state.from_rev, "roots((newest-commit) | (oldest-commit))-");
        assert_eq!(state.to_rev, "heads((newest-commit) | (oldest-commit))");
        assert_eq!(state.target_change_id.as_deref(), Some("newest-commit"));
        assert_eq!(state.display.title, "2 Changes Selected");
        assert_eq!(state.display.from, "oldest");
        assert_eq!(state.display.to, "newest");
        assert!(state.display.is_combined_selection);
    }

    #[test]
    fn a_selection_without_two_distinct_revisions_is_not_a_range() {
        assert!(combined_diff_revsets(&["only".to_owned()]).is_none());
        assert!(combined_diff_revsets(&["same".to_owned(), " same ".to_owned()]).is_none());
        assert!(CompareState::combined(&[change("only", &[])]).is_none());
    }

    #[test]
    fn revset_ends_label_themselves_when_no_change_is_visible() {
        let display = CompareDisplay::for_revsets("\"main\"", "fork_point(a | b)", &[]);

        assert_eq!(display.title, "Comparing");
        assert_eq!(display.from, "main");
        assert_eq!(display.to, "fork_point(a | b)");
        assert!(!display.is_combined_selection);
    }

    #[test]
    fn a_revision_that_names_no_visible_change_labels_itself() {
        let changes = [change("change-id-long", &["main"])];

        assert_eq!(revision_label("change-id-long", &changes), "main");
        assert_eq!(revision_label("change-id-long-commit", &changes), "main");
        assert_eq!(revision_label("\"feature\"", &changes), "feature");
        assert_eq!(
            revision_label("fork_point(a | b)", &changes),
            "fork_point(a | b)"
        );
        assert_eq!(revision_label("abcdefghijkl", &changes), "abcdefgh");
    }
}
