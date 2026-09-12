use std::collections::{HashMap, HashSet};

use crate::types::{EdgeType, GraphEntry};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SelectionState {
    pub can_abandon: bool,
    pub can_squash: bool,
    /// The combined diff bases on `roots(selection)-`, so the oldest change needs exactly one parent.
    pub can_diff: bool,
    pub can_merge: bool,
    /// Indexed by graph row, as is `can_merge_with`.
    pub can_rebase_onto: Vec<bool>,
    pub can_merge_with: Vec<bool>,
}

#[derive(Debug)]
pub struct SelectionGraph {
    rows: Vec<Row>,
    children: Vec<Vec<usize>>,
    row_by_commit_id: HashMap<String, usize>,
}

#[derive(Debug)]
struct Row {
    commit_id: String,
    /// Row indices; edges that leave the page are not followed.
    parents: Vec<usize>,
    /// As jj reports them, including parents this page does not show.
    parent_commit_ids: Vec<String>,
    is_immutable: bool,
}

impl SelectionGraph {
    pub fn new(entries: &[GraphEntry]) -> Self {
        let row_by_commit_id: HashMap<String, usize> = entries
            .iter()
            .enumerate()
            .map(|(ix, entry)| (entry.change.commit_id.id.clone(), ix))
            .collect();
        let rows: Vec<Row> = entries
            .iter()
            .map(|entry| Row {
                commit_id: entry.change.commit_id.id.clone(),
                parents: entry
                    .edges
                    .iter()
                    .filter(|edge| edge.edge_type != EdgeType::Missing)
                    .filter_map(|edge| row_by_commit_id.get(&edge.target).copied())
                    .collect(),
                parent_commit_ids: entry.change.parents.clone(),
                is_immutable: entry.change.is_immutable,
            })
            .collect();
        let mut children = vec![Vec::new(); rows.len()];
        for (ix, row) in rows.iter().enumerate() {
            for &parent in &row.parents {
                children[parent].push(ix);
            }
        }
        Self {
            rows,
            children,
            row_by_commit_id,
        }
    }

    pub fn state(&self, selected_commit_ids: &[String]) -> SelectionState {
        let unique: HashSet<&str> = selected_commit_ids.iter().map(String::as_str).collect();
        let selected: HashSet<usize> = unique
            .iter()
            .filter_map(|id| self.row_by_commit_id.get(*id).copied())
            .collect();
        // A selection naming changes this page does not show cannot be acted on as a whole.
        if selected.is_empty() || selected.len() != unique.len() {
            return SelectionState {
                can_rebase_onto: vec![false; self.rows.len()],
                can_merge_with: vec![false; self.rows.len()],
                ..SelectionState::default()
            };
        }

        let mut ordered: Vec<usize> = selected.iter().copied().collect();
        ordered.sort_unstable();
        let ancestors = self.reachable(&ordered, &|ix| &self.rows[ix].parents);
        let descendants = self.reachable(&ordered, &|ix| &self.children[ix]);

        let mutable = ordered.len() > 1 && ordered.iter().all(|&ix| !self.rows[ix].is_immutable);
        let contiguous = ordered.len() > 1
            && ordered[ordered.len() - 1] - ordered[0] + 1 == ordered.len()
            && ordered
                .windows(2)
                .all(|pair| self.is_only_parent(pair[0], pair[1]));
        let oldest_has_one_parent = ordered
            .last()
            .is_some_and(|&ix| self.rows[ix].parent_commit_ids.len() == 1);
        // Merge parents must be independent heads: no selected change may be an ancestor of another.
        let can_merge = ordered.len() > 1 && !ordered.iter().any(|ix| ancestors.contains(ix));

        SelectionState {
            can_abandon: mutable,
            can_squash: mutable && contiguous,
            can_diff: contiguous && oldest_has_one_parent,
            can_merge,
            can_rebase_onto: (0..self.rows.len())
                .map(|ix| mutable && !selected.contains(&ix) && !descendants.contains(&ix))
                .collect(),
            can_merge_with: (0..self.rows.len())
                .map(|ix| {
                    (ordered.len() == 1 || can_merge)
                        && !selected.contains(&ix)
                        && !ancestors.contains(&ix)
                        && !descendants.contains(&ix)
                })
                .collect(),
        }
    }

    fn is_only_parent(&self, child: usize, parent: usize) -> bool {
        let parent_ids = &self.rows[child].parent_commit_ids;
        parent_ids.len() == 1 && parent_ids[0] == self.rows[parent].commit_id
    }

    fn reachable<'a>(
        &'a self,
        from: &[usize],
        links: &dyn Fn(usize) -> &'a Vec<usize>,
    ) -> HashSet<usize> {
        let mut pending: Vec<usize> = from
            .iter()
            .flat_map(|&ix| links(ix).iter().copied())
            .collect();
        let mut visited = HashSet::new();
        while let Some(ix) = pending.pop() {
            if visited.insert(ix) {
                pending.extend(links(ix).iter().copied());
            }
        }
        visited
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::entry;
    use super::*;

    /// d — c — b — a, plus a sibling head `x` on `c`, and `i` is immutable.
    fn fixture() -> Vec<GraphEntry> {
        let mut entries = vec![
            entry("d", &["c"]),
            entry("x", &["c"]),
            entry("c", &["b"]),
            entry("b", &["a"]),
            entry("a", &[]),
            entry("i", &[]),
        ];
        entries[5].change.is_immutable = true;
        entries
    }

    fn state(selected: &[&str]) -> SelectionState {
        let selected: Vec<String> = selected.iter().map(|id| (*id).to_owned()).collect();
        SelectionGraph::new(&fixture()).state(&selected)
    }

    fn row(id: &str) -> usize {
        fixture()
            .iter()
            .position(|entry| entry.change.commit_id.id == id)
            .expect("row")
    }

    #[test]
    fn a_parent_linked_range_can_be_squashed_abandoned_and_diffed() {
        let range = state(&["c", "b"]);
        assert!(range.can_abandon);
        assert!(range.can_squash);
        assert!(range.can_diff);
        assert!(
            !range.can_merge,
            "one selected change is the other's ancestor"
        );
    }

    #[test]
    fn a_range_reaching_the_root_has_no_combined_diff_base() {
        let to_root = state(&["b", "a"]);
        assert!(to_root.can_squash, "squashing into the root is still legal");
        assert!(
            !to_root.can_diff,
            "roots(selection)- needs a parent to base on"
        );
    }

    #[test]
    fn a_gap_or_a_fork_is_neither_squashable_nor_diffable() {
        let gap = state(&["d", "b"]);
        assert!(!gap.can_squash);
        assert!(!gap.can_diff);
        assert!(gap.can_abandon, "abandoning does not need a range");

        let heads = state(&["d", "x"]);
        assert!(
            !heads.can_squash,
            "siblings are adjacent but not parent-linked"
        );
        assert!(heads.can_merge, "independent heads are what merge wants");
    }

    #[test]
    fn an_immutable_or_unloaded_change_blocks_the_whole_selection() {
        let immutable = state(&["c", "i"]);
        assert!(!immutable.can_abandon);
        assert!(!immutable.can_squash);
        assert!(
            !immutable.can_rebase_onto[row("a")],
            "an immutable selection has nowhere to rebase"
        );

        let off_page = state(&["c", "not-loaded"]);
        assert!(!off_page.can_abandon);
        assert!(!off_page.can_merge);
        assert!(off_page.can_merge_with.iter().all(|allowed| !allowed));
    }

    #[test]
    fn rebase_refuses_the_selection_and_its_descendants() {
        let selection = state(&["c", "b"]);
        assert!(
            selection.can_rebase_onto[row("a")],
            "an ancestor is a valid destination"
        );
        assert!(
            selection.can_rebase_onto[row("i")],
            "an unrelated head is too"
        );
        assert!(!selection.can_rebase_onto[row("d")], "a descendant is not");
        assert!(
            !selection.can_rebase_onto[row("x")],
            "nor is another descendant"
        );
        assert!(
            !selection.can_rebase_onto[row("c")],
            "nor is the selection itself"
        );
    }

    #[test]
    fn an_edge_that_leaves_the_page_carries_no_ancestry() {
        let mut entries = vec![entry("child", &["off-page"]), entry("other", &[])];
        entries[0].edges[0].edge_type = EdgeType::Missing;
        entries[0].edges.push(crate::types::GraphEdge {
            target: "other".to_owned(),
            edge_type: EdgeType::Indirect,
        });
        let graph = SelectionGraph::new(&entries);

        let state = graph.state(&["child".to_owned()]);
        assert!(
            !state.can_merge_with[1],
            "an indirect edge still means the row is an ancestor"
        );

        entries[0].edges.pop();
        let state = SelectionGraph::new(&entries).state(&["child".to_owned()]);
        assert!(
            state.can_merge_with[1],
            "with only the missing edge left there is no path between them"
        );
    }

    #[test]
    fn merge_refuses_a_row_on_either_side_of_the_selection() {
        let selection = state(&["c"]);
        assert!(!selection.can_merge, "one change cannot merge with itself");
        assert!(selection.can_merge_with[row("i")], "an unrelated head can");
        assert!(!selection.can_merge_with[row("a")], "an ancestor cannot");
        assert!(!selection.can_merge_with[row("d")], "a descendant cannot");
        assert!(!selection.can_merge_with[row("c")], "nor itself");
    }
}
