use std::collections::HashSet;

use crate::types::{EdgeType, GraphEntry};

/// jj rejects a rebase onto a descendant, so refuse it before the confirmation sheet; self and the only parent would move nothing.
pub fn can_rebase_onto(
    entries: &[GraphEntry],
    source_commit_id: &str,
    target_commit_id: &str,
) -> bool {
    if source_commit_id == target_commit_id {
        return false;
    }
    let Some(source) = entries
        .iter()
        .find(|entry| entry.change.commit_id.id == source_commit_id)
    else {
        return false;
    };
    if source.change.parents == [target_commit_id] {
        return false;
    }
    !has_ancestor(entries, target_commit_id, source_commit_id)
}

/// Every visible descendant of `commit_id`; rows are in graph order (children above parents), so only the rows above it can descend from it.
pub fn descendant_commit_ids(entries: &[GraphEntry], commit_id: &str) -> Vec<String> {
    let Some(source) = entries
        .iter()
        .position(|entry| entry.change.commit_id.id == commit_id)
    else {
        return Vec::new();
    };
    let mut descendants = HashSet::from([commit_id]);
    for entry in entries[..source].iter().rev() {
        let reaches = entry
            .edges
            .iter()
            .filter(|edge| edge.edge_type != EdgeType::Missing)
            .any(|edge| descendants.contains(edge.target.as_str()));
        if reaches {
            descendants.insert(entry.change.commit_id.id.as_str());
        }
    }
    descendants.remove(commit_id);
    descendants.into_iter().map(str::to_owned).collect()
}

/// Rows are in graph order (children above parents), so a descendant sits above its ancestor and only the rows between them can connect the two.
fn has_ancestor(entries: &[GraphEntry], commit_id: &str, ancestor_id: &str) -> bool {
    let (mut start, mut end) = (None, None);
    for (ix, entry) in entries.iter().enumerate() {
        let id = entry.change.commit_id.id.as_str();
        if id == commit_id {
            start = Some(ix);
        } else if id == ancestor_id {
            end = Some(ix);
        }
        if start.is_some() && end.is_some() {
            break;
        }
    }
    let (Some(start), Some(end)) = (start, end) else {
        return false;
    };
    if end <= start {
        return false;
    }
    let mut reachable = HashSet::from([commit_id]);
    for entry in &entries[start..end] {
        if reachable.contains(entry.change.commit_id.id.as_str()) {
            reachable.extend(
                entry
                    .edges
                    .iter()
                    .filter(|edge| edge.edge_type != EdgeType::Missing)
                    .map(|edge| edge.target.as_str()),
            );
        }
    }
    reachable.contains(ancestor_id)
}

#[cfg(test)]
mod tests {
    use super::super::tests::entry;
    use super::*;

    #[test]
    fn a_change_refuses_itself_its_only_parent_and_its_descendants() {
        let entries = [
            entry("a", &["b"]),
            entry("d", &["c"]),
            entry("b", &["c"]),
            entry("c", &[]),
        ];

        assert!(!can_rebase_onto(&entries, "b", "b"), "self");
        assert!(!can_rebase_onto(&entries, "b", "c"), "only parent");
        assert!(!can_rebase_onto(&entries, "b", "a"), "descendant");
        assert!(can_rebase_onto(&entries, "b", "d"), "sibling branch");
    }

    #[test]
    fn descendants_are_every_row_that_leads_back_to_the_change() {
        let entries = [
            entry("a", &["b"]),
            entry("d", &["c"]),
            entry("b", &["c"]),
            entry("c", &[]),
        ];
        let sorted = |id: &str| {
            let mut ids = descendant_commit_ids(&entries, id);
            ids.sort();
            ids
        };

        assert_eq!(sorted("c"), ["a", "b", "d"]);
        assert_eq!(sorted("b"), ["a"]);
        assert!(sorted("a").is_empty());
        assert!(sorted("missing").is_empty());
    }

    #[test]
    fn an_indirect_descendant_is_still_refused_and_an_ancestor_is_allowed() {
        let entries = [entry("a", &["b"]), entry("b", &["c"]), entry("c", &[])];

        assert!(!can_rebase_onto(&entries, "c", "a"));
        assert!(can_rebase_onto(&entries, "a", "c"), "grandparent");
    }
}
