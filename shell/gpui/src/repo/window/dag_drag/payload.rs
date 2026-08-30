use std::collections::HashSet;
use std::sync::Arc;

use jayjay_core::{ChangeInfo, EdgeType, GraphEntry};

#[derive(Clone)]
pub(crate) enum DagDrag {
    WorkingCopy,
    Bookmark {
        name: String,
        conflicted: bool,
    },
    Change {
        source_ix: usize,
        entries: Arc<Vec<GraphEntry>>,
    },
}

impl DagDrag {
    pub(in crate::repo::window) fn for_change(
        ix: usize,
        entries: &Arc<Vec<GraphEntry>>,
    ) -> Option<Self> {
        (!entries.get(ix)?.change.is_immutable).then(|| Self::Change {
            source_ix: ix,
            entries: entries.clone(),
        })
    }

    pub(super) fn source_change(&self) -> Option<&ChangeInfo> {
        match self {
            Self::Change { source_ix, entries } => {
                entries.get(*source_ix).map(|entry| &entry.change)
            }
            _ => None,
        }
    }

    pub(super) fn label_for_change(change: &ChangeInfo) -> String {
        change
            .bookmarks
            .first()
            .filter(|bookmark| !bookmark.is_empty())
            .cloned()
            .or_else(|| {
                change
                    .description
                    .lines()
                    .next()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(str::to_owned)
            })
            .or_else(|| change.is_working_copy.then(|| "@".to_owned()))
            .unwrap_or_else(|| change.change_id.prefix(8))
    }

    pub(in crate::repo::window) fn can_drop_on(&self, change: &ChangeInfo) -> bool {
        match self {
            Self::WorkingCopy => !change.is_working_copy,
            Self::Bookmark { name, conflicted } => {
                *conflicted || !change.bookmarks.iter().any(|bookmark| bookmark == name)
            }
            Self::Change { source_ix, entries } => {
                let Some(source) = self.source_change() else {
                    return false;
                };
                let target_id = change.commit_id.as_str();
                let source_id = source.commit_id.as_str();
                target_id != source_id
                    && source.parents.as_slice() != [target_id]
                    && !descends_from(&entries[..*source_ix], target_id, source_id)
            }
        }
    }
}

/// Rows are in graph order (children above parents), so a descendant of the source sits in the rows above it; follow the displayed edges down from the target.
fn descends_from(rows_above: &[GraphEntry], target_id: &str, source_id: &str) -> bool {
    let Some(target_ix) = rows_above
        .iter()
        .position(|entry| entry.change.commit_id.as_str() == target_id)
    else {
        return false;
    };
    let mut reachable = HashSet::from([target_id]);
    for entry in &rows_above[target_ix..] {
        if reachable.contains(entry.change.commit_id.as_str()) {
            reachable.extend(
                entry
                    .edges
                    .iter()
                    .filter(|edge| edge.edge_type != EdgeType::Missing)
                    .map(|edge| edge.target.as_str()),
            );
        }
    }
    reachable.contains(source_id)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use jayjay_core::{ChangeInfo, CommitAuthor, EdgeType, GraphEdge, GraphEntry, ShortId};

    use super::DagDrag;

    fn entry(commit: &str, edges: &[&str]) -> GraphEntry {
        let change = ChangeInfo {
            change_id: ShortId::new(format!("change-{commit}"), 1),
            commit_id: ShortId::new(commit.to_owned(), 1),
            description: String::new(),
            author: CommitAuthor::empty(0),
            parents: edges.iter().map(|edge| (*edge).to_owned()).collect(),
            bookmarks: Vec::new(),
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
        };
        GraphEntry {
            change,
            edges: edges
                .iter()
                .map(|target| GraphEdge {
                    target: (*target).to_owned(),
                    edge_type: EdgeType::Direct,
                })
                .collect(),
        }
    }

    #[test]
    fn change_drag_refuses_itself_its_only_parent_and_its_descendants() {
        let entries = Arc::new(vec![
            entry("a", &["b"]),
            entry("d", &["c"]),
            entry("b", &["c"]),
            entry("c", &[]),
        ]);
        let drag = DagDrag::for_change(2, &entries).expect("b is draggable");
        let target = |ix: usize| &entries[ix].change;

        assert!(!drag.can_drop_on(target(2)), "self");
        assert!(!drag.can_drop_on(target(3)), "only parent");
        assert!(!drag.can_drop_on(target(0)), "descendant");
        assert!(drag.can_drop_on(target(1)), "sibling branch");
    }
}
