use std::collections::{HashMap, HashSet};

use super::DagLayout;
use crate::types::{EdgeType, GraphEntry};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SelectionState {
    pub can_abandon: bool,
    pub can_squash: bool,
    /// The combined diff bases on `roots(selection)-`, so the oldest change needs exactly one parent.
    pub can_diff: bool,
    pub can_merge: bool,
    pub can_parallelize: bool,
    pub can_fix: bool,
    /// Indexed by graph row, as is `can_merge_with`.
    pub can_rebase_onto: Vec<bool>,
    pub can_merge_with: Vec<bool>,
}

/// The rows a lane step can reach from a selection without leaving its lane, top to bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaneSpan {
    pub lane: usize,
    pub first_row: usize,
    pub last_row: usize,
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
    lane: usize,
}

impl SelectionGraph {
    pub fn new(entries: &[GraphEntry], layout: &DagLayout) -> Self {
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
                lane: layout.lane(&entry.change.commit_id.id),
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

        let all_mutable = ordered.iter().all(|&ix| !self.rows[ix].is_immutable);
        let mutable = ordered.len() > 1 && all_mutable;
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
        // Parallelize needs one connected run: each selected change descended from the next, possibly through unselected changes.
        let can_parallelize = mutable
            && ordered
                .windows(2)
                .all(|pair| self.reaches(pair[0], pair[1]));

        SelectionState {
            can_abandon: mutable,
            can_squash: mutable && contiguous,
            can_diff: contiguous && oldest_has_one_parent,
            can_merge,
            can_parallelize,
            can_fix: all_mutable,
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

    /// Keyboard step down a lane: the parent drawn in the row's lane, else the nearest parent below, else the next row so disconnected results stay reachable.
    pub fn lane_parent(&self, row: usize) -> Option<usize> {
        let parents = &self.rows.get(row)?.parents;
        self.in_lane_of(row, parents)
            .or_else(|| parents.iter().copied().min())
            .or_else(|| (row + 1 < self.rows.len()).then_some(row + 1))
    }

    /// Keyboard step up a lane: the child drawn in the row's lane, else the nearest child above, else the previous row.
    pub fn lane_child(&self, row: usize) -> Option<usize> {
        let children = self.children.get(row)?;
        self.in_lane_of(row, children)
            .or_else(|| children.iter().copied().max())
            .or_else(|| row.checked_sub(1))
    }

    /// Keyboard step to the nearest lane on the left, landing on its row closest to `row`.
    pub fn lane_left(&self, row: usize) -> Option<usize> {
        self.across_lanes(row, false)
    }

    /// Keyboard step to the nearest lane on the right, landing on its row closest to `row`.
    pub fn lane_right(&self, row: usize) -> Option<usize> {
        self.across_lanes(row, true)
    }

    pub fn lane_span(&self, row: usize) -> Option<LaneSpan> {
        let lane = self.rows.get(row)?.lane;
        let mut first_row = row;
        while let Some(child) = self.in_lane_of(first_row, &self.children[first_row]) {
            first_row = child;
        }
        let mut last_row = row;
        while let Some(parent) = self.in_lane_of(last_row, &self.rows[last_row].parents) {
            last_row = parent;
        }
        Some(LaneSpan {
            lane,
            first_row,
            last_row,
        })
    }

    fn across_lanes(&self, row: usize, rightward: bool) -> Option<usize> {
        let lane = self.rows.get(row)?.lane;
        let target = self
            .rows
            .iter()
            .map(|candidate| candidate.lane)
            .filter(|&other| {
                if rightward {
                    other > lane
                } else {
                    other < lane
                }
            })
            .min_by_key(|other| other.abs_diff(lane))?;
        (0..self.rows.len())
            .filter(|&ix| self.rows[ix].lane == target)
            .min_by_key(|ix| ix.abs_diff(row))
    }

    fn in_lane_of(&self, row: usize, candidates: &[usize]) -> Option<usize> {
        let lane = self.rows[row].lane;
        candidates
            .iter()
            .copied()
            .find(|&ix| self.rows[ix].lane == lane)
    }

    fn reaches(&self, child: usize, ancestor: usize) -> bool {
        self.reachable(&[child], &|ix| &self.rows[ix].parents)
            .contains(&ancestor)
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
    use super::*;
    use crate::mock::graph_entry;

    fn graph(entries: &[GraphEntry]) -> SelectionGraph {
        SelectionGraph::new(entries, &DagLayout::compute(entries))
    }

    /// d — c — b — a, plus a sibling head `x` on `c`, and `i` is immutable.
    fn fixture() -> Vec<GraphEntry> {
        let mut entries = vec![
            graph_entry("d", &["c"]),
            graph_entry("x", &["c"]),
            graph_entry("c", &["b"]),
            graph_entry("b", &["a"]),
            graph_entry("a", &[]),
            graph_entry("i", &[]),
        ];
        entries[5].change.is_immutable = true;
        entries
    }

    fn state(selected: &[&str]) -> SelectionState {
        let selected: Vec<String> = selected.iter().map(|id| (*id).to_owned()).collect();
        graph(&fixture()).state(&selected)
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
        assert!(range.can_fix && state(&["c"]).can_fix);
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
        assert!(!immutable.can_fix && !state(&["i"]).can_fix);
        assert!(
            !immutable.can_rebase_onto[row("a")],
            "an immutable selection has nowhere to rebase"
        );

        let off_page = state(&["c", "not-loaded"]);
        assert!(!off_page.can_abandon);
        assert!(!off_page.can_merge);
        assert!(!off_page.can_fix);
        let every_row_refused = vec![false; fixture().len()];
        assert_eq!(off_page.can_rebase_onto, every_row_refused);
        assert_eq!(off_page.can_merge_with, every_row_refused);
    }

    #[test]
    fn a_chain_of_changes_can_be_parallelized_but_a_fork_cannot() {
        let range = state(&["c", "b"]);
        assert!(range.can_parallelize, "a parent-linked range is connected");

        let gap = state(&["d", "b"]);
        assert!(
            gap.can_parallelize,
            "a run connected through an unselected change stays eligible"
        );

        let heads = state(&["d", "x"]);
        assert!(!heads.can_parallelize, "sibling heads are already parallel");

        let single = state(&["c"]);
        assert!(!single.can_parallelize, "one change is not a run");

        let immutable = state(&["c", "i"]);
        assert!(!immutable.can_parallelize);
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
    fn lane_steps_follow_and_cross_the_drawn_lanes() {
        let graph = graph(&[
            graph_entry("s2", &["s1"]),
            graph_entry("s1", &["trunk"]),
            graph_entry("t2", &["t1"]),
            graph_entry("t1", &["trunk"]),
            graph_entry("trunk", &[]),
        ]);

        assert_eq!(graph.lane_parent(1), Some(4));
        assert_eq!(graph.lane_child(4), Some(1));
        assert_eq!(graph.lane_parent(3), Some(4));
        assert_eq!(graph.lane_child(2), Some(1));
        assert_eq!(graph.lane_parent(4), None);

        let entries = [graph_entry("h1", &[]), graph_entry("h2", &[])];
        let heads = SelectionGraph::new(&entries, &DagLayout::compute(&entries));
        assert_eq!(
            (heads.lane_parent(0), heads.lane_child(1)),
            (Some(1), Some(0))
        );

        assert_eq!(graph.lane_right(1), Some(2));
        assert_eq!(graph.lane_right(3), None);
        assert_eq!(graph.lane_left(3), Some(4));
        assert_eq!(graph.lane_left(0), None);

        let span = |row| {
            graph
                .lane_span(row)
                .map(|s| (s.lane, s.first_row, s.last_row))
        };
        assert_eq!(span(3), Some((1, 2, 3)));
        assert_eq!(span(1), Some((0, 0, 4)));
    }

    #[test]
    fn an_edge_that_leaves_the_page_carries_no_ancestry() {
        let mut entries = vec![
            graph_entry("child", &["off-page"]),
            graph_entry("other", &[]),
        ];
        entries[0].edges[0].edge_type = EdgeType::Missing;
        entries[0].edges.push(crate::types::GraphEdge {
            target: "other".to_owned(),
            edge_type: EdgeType::Indirect,
        });
        let state = graph(&entries).state(&["child".to_owned()]);
        assert!(
            !state.can_merge_with[1],
            "an indirect edge still means the row is an ancestor"
        );

        entries[0].edges.pop();
        let state = graph(&entries).state(&["child".to_owned()]);
        assert!(
            state.can_merge_with[1],
            "with only the missing edge left there is no path between them"
        );
    }

    #[test]
    fn merge_refuses_a_row_on_either_side_of_the_selection() {
        let selection = state(&["c"]);
        assert!(!selection.can_abandon);
        assert!(!selection.can_squash);
        assert!(!selection.can_diff);
        assert!(selection.can_rebase_onto.iter().all(|allowed| !allowed));
        assert!(!selection.can_merge, "one change cannot merge with itself");
        assert!(selection.can_merge_with[row("i")], "an unrelated head can");
        assert!(!selection.can_merge_with[row("a")], "an ancestor cannot");
        assert!(!selection.can_merge_with[row("d")], "a descendant cannot");
        assert!(!selection.can_merge_with[row("c")], "nor itself");
    }
}
