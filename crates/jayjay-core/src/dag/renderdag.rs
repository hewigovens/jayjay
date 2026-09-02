//! Adapter from the `sapling-renderdag` crate's `GraphRowRenderer` to JayJay's app-owned row shapes.
//!
//! No upstream type crosses this module boundary.

use std::collections::HashMap;

use renderdag::{Ancestor, GraphRow, GraphRowRenderer, LinkLine, NodeLine, PadLine, Renderer};

use super::row_shape::{DagEdgeKind, DagLayout, DagLinkCell, DagRowShape, DagVerticalCell};
use crate::types::{EdgeType, GraphEntry};

impl DagLayout {
    pub fn compute(entries: &[GraphEntry]) -> Self {
        let mut renderer = GraphRowRenderer::<String>::new();
        let mut rows = Vec::with_capacity(entries.len());
        let mut logical_column_count = 0;
        let mut incoming = HashMap::new();

        for entry in entries {
            let commit_id = entry.change.commit_id.id.clone();
            let parents = entry.edges.iter().map(to_ancestor).collect();
            let row = renderer.next_row(commit_id.clone(), parents, String::new(), String::new());
            debug_assert_eq!(
                row.node, commit_id,
                "renderer emitted a row for a different commit than requested"
            );
            logical_column_count = logical_column_count.max(row.node_line.len() as u32);
            rows.push(to_row_shape(row, incoming.remove(&commit_id)));
            register_incoming_edges(&mut incoming, entry);
        }

        DagLayout {
            rows,
            logical_column_count,
        }
    }
}

fn register_incoming_edges(incoming: &mut HashMap<String, DagEdgeKind>, entry: &GraphEntry) {
    for edge in &entry.edges {
        let kind = match edge.edge_type {
            EdgeType::Direct => DagEdgeKind::Direct,
            EdgeType::Indirect => DagEdgeKind::Indirect,
            EdgeType::Missing => continue,
        };
        incoming
            .entry(edge.target.clone())
            .and_modify(|current| {
                if kind == DagEdgeKind::Direct {
                    *current = kind;
                }
            })
            .or_insert(kind);
    }
}

fn to_ancestor(edge: &crate::types::GraphEdge) -> Ancestor<String> {
    match edge.edge_type {
        EdgeType::Direct => Ancestor::Parent(edge.target.clone()),
        EdgeType::Indirect => Ancestor::Ancestor(edge.target.clone()),
        EdgeType::Missing => Ancestor::Anonymous,
    }
}

fn to_row_shape(row: GraphRow<String>, incoming: Option<DagEdgeKind>) -> DagRowShape {
    let node_column = node_column(&row.node_line);
    let node_line = row.node_line.iter().map(to_vertical_cell).collect();
    let pad_line = row.pad_lines.iter().map(pad_to_vertical_cell).collect();
    let link_line = row
        .link_line
        .map(|cells| cells.iter().map(to_link_cell).collect());
    let termination_columns = row
        .term_line
        .map(|flags| {
            flags
                .iter()
                .enumerate()
                .filter_map(|(column, &terminates)| terminates.then_some(column as u32))
                .collect()
        })
        .unwrap_or_default();

    DagRowShape {
        commit_id: row.node,
        node_column,
        incoming,
        node_line,
        link_line,
        termination_columns,
        pad_line,
    }
}

/// The single column carrying the node glyph. The renderer emits exactly one `NodeLine::Node` per row; anything else is a renderer-contract violation, not runtime data, so we surface it in debug builds and fall back to the node column in release.
fn node_column(node_line: &[NodeLine]) -> u32 {
    let mut nodes = node_line
        .iter()
        .enumerate()
        .filter(|(_, cell)| **cell == NodeLine::Node)
        .map(|(column, _)| column as u32);
    let column = nodes.next();
    debug_assert!(column.is_some(), "renderer row has no node cell");
    debug_assert!(
        nodes.next().is_none(),
        "renderer row has more than one node cell"
    );
    column.unwrap_or(0)
}

fn to_vertical_cell(line: &NodeLine) -> DagVerticalCell {
    match line {
        NodeLine::Blank | NodeLine::Node => DagVerticalCell::Empty,
        NodeLine::Parent => DagVerticalCell::Direct,
        NodeLine::Ancestor => DagVerticalCell::Indirect,
    }
}

fn pad_to_vertical_cell(line: &PadLine) -> DagVerticalCell {
    match line {
        PadLine::Blank => DagVerticalCell::Empty,
        PadLine::Parent => DagVerticalCell::Direct,
        PadLine::Ancestor => DagVerticalCell::Indirect,
    }
}

fn to_link_cell(flags: &LinkLine) -> DagLinkCell {
    DagLinkCell {
        vertical: edge_kind(*flags, LinkLine::VERT_PARENT, LinkLine::VERT_ANCESTOR),
        horizontal: edge_kind(*flags, LinkLine::HORIZ_PARENT, LinkLine::HORIZ_ANCESTOR),
        left_fork: edge_kind(
            *flags,
            LinkLine::LEFT_FORK_PARENT,
            LinkLine::LEFT_FORK_ANCESTOR,
        ),
        right_fork: edge_kind(
            *flags,
            LinkLine::RIGHT_FORK_PARENT,
            LinkLine::RIGHT_FORK_ANCESTOR,
        ),
        left_merge: edge_kind(
            *flags,
            LinkLine::LEFT_MERGE_PARENT,
            LinkLine::LEFT_MERGE_ANCESTOR,
        ),
        right_merge: edge_kind(
            *flags,
            LinkLine::RIGHT_MERGE_PARENT,
            LinkLine::RIGHT_MERGE_ANCESTOR,
        ),
        is_child: flags.contains(LinkLine::CHILD),
    }
}

fn edge_kind(flags: LinkLine, direct: LinkLine, indirect: LinkLine) -> Option<DagEdgeKind> {
    if flags.contains(direct) {
        Some(DagEdgeKind::Direct)
    } else if flags.contains(indirect) {
        Some(DagEdgeKind::Indirect)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChangeInfo, CommitAuthor, GraphEdge, NewChangeEligibility, ShortId};

    fn entry(commit_id: &str, edges: &[(&str, EdgeType)]) -> GraphEntry {
        GraphEntry {
            change: ChangeInfo {
                change_id: ShortId::new(format!("change-{commit_id}"), 1),
                commit_id: ShortId::new(commit_id.to_owned(), 1),
                description: String::new(),
                author: CommitAuthor::empty(0),
                parents: edges.iter().map(|(p, _)| (*p).to_owned()).collect(),
                bookmarks: Vec::new(),
                tags: Vec::new(),
                workspaces: Vec::new(),
                is_working_copy: false,
                has_conflict: false,
                is_empty: false,
                is_immutable: false,
                is_divergent: false,
                new_change: NewChangeEligibility {
                    on_top: true,
                    before: true,
                    after: true,
                },
            },
            edges: edges
                .iter()
                .map(|(target, edge_type)| GraphEdge {
                    target: (*target).to_owned(),
                    edge_type: *edge_type,
                })
                .collect(),
        }
    }

    fn direct(commit_id: &str, parents: &[&str]) -> GraphEntry {
        entry(
            commit_id,
            &parents
                .iter()
                .map(|p| (*p, EdgeType::Direct))
                .collect::<Vec<_>>(),
        )
    }

    fn columns(layout: &DagLayout, commit_id: &str) -> u32 {
        layout
            .row(commit_id)
            .unwrap_or_else(|| panic!("no row for {commit_id}"))
            .node_column
    }

    #[test]
    fn linear_history_stays_in_one_column() {
        let entries = vec![direct("C", &["B"]), direct("B", &["A"]), direct("A", &[])];
        let layout = DagLayout::compute(&entries);

        assert_eq!(layout.logical_column_count, 1);
        assert_eq!(columns(&layout, "C"), 0);
        assert_eq!(columns(&layout, "B"), 0);
        assert_eq!(columns(&layout, "A"), 0);
    }

    #[test]
    fn disconnected_heads_reuse_the_freed_column() {
        let entries = vec![direct("D", &[]), direct("C", &[])];
        let layout = DagLayout::compute(&entries);

        // C's row is emitted after D's column is freed, so it reuses column 0 rather than opening a second lane — matching `jj log`'s own output for two adjacent, unrelated heads.
        assert_eq!(columns(&layout, "D"), 0);
        assert_eq!(columns(&layout, "C"), 0);
        assert_eq!(layout.logical_column_count, 1);
    }

    #[test]
    fn fork_then_merge_matches_renderer_column_transitions() {
        // D forks into B and C, both reconverge on A.
        let entries = vec![
            direct("D", &["B", "C"]),
            direct("B", &["A"]),
            direct("C", &["A"]),
            direct("A", &[]),
        ];
        let layout = DagLayout::compute(&entries);

        assert_eq!(columns(&layout, "D"), 0);
        assert_eq!(columns(&layout, "B"), 0);
        assert_eq!(columns(&layout, "C"), 1);
        assert_eq!(columns(&layout, "A"), 0);
        assert_eq!(layout.logical_column_count, 2);

        let d_row = &layout.rows[0];
        let link = d_row
            .link_line
            .as_ref()
            .expect("fork row needs a link line");
        assert!(
            link[1].left_fork.is_some(),
            "column 1 should fork left toward D"
        );
    }

    #[test]
    fn octopus_merge_keeps_every_parent_column() {
        let entries = vec![
            direct("merge", &["p0", "p1", "p2", "p3", "p4", "p5"]),
            direct("p5", &[]),
            direct("p4", &[]),
            direct("p3", &[]),
            direct("p2", &[]),
            direct("p1", &[]),
            direct("p0", &[]),
        ];
        let layout = DagLayout::compute(&entries);

        assert_eq!(layout.logical_column_count, 6);
        assert_eq!(columns(&layout, "p0"), 0);
        assert_eq!(columns(&layout, "p5"), 5);
    }

    #[test]
    fn terminal_octopus_merge_counts_columns_created_below_the_node_line() {
        let layout = DagLayout::compute(&[direct("merge", &["p0", "p1", "p2", "p3", "p4", "p5"])]);

        assert_eq!(layout.rows[0].node_line.len(), 6);
        assert_eq!(layout.rows[0].pad_line.len(), 6);
        assert_eq!(layout.logical_column_count, 6);
    }

    #[test]
    fn interleaved_heads_reuse_columns_per_renderer_not_first_free_heuristic() {
        // Two independent forks interleaved: X forks x0/x1, then Y forks y0/y1, both x0 and y0 free their columns before the other fork's second branch lands.
        let entries = vec![
            direct("X", &["x0", "x1"]),
            direct("x0", &[]),
            direct("Y", &["y0", "y1"]),
            direct("y0", &[]),
            direct("x1", &[]),
            direct("y1", &[]),
        ];
        let layout = DagLayout::compute(&entries);

        assert_eq!(columns(&layout, "X"), 0);
        assert_eq!(columns(&layout, "x0"), 0);
        assert_eq!(columns(&layout, "Y"), 0);
        // y0 reuses column 0, freed by x0; y1 keeps the column X's second parent left open.
        assert_eq!(columns(&layout, "y0"), 0);
        assert_eq!(columns(&layout, "x1"), 1);
        assert_eq!(columns(&layout, "y1"), 2);
    }

    #[test]
    fn single_parent_lane_move_is_expressed_by_the_link_line() {
        let entries = vec![
            direct("X", &["A", "P"]),
            direct("A", &[]),
            direct("B", &["P"]),
            direct("P", &[]),
        ];

        let layout = DagLayout::compute(&entries);
        let link = layout.rows[2]
            .link_line
            .as_ref()
            .expect("moving P from column 1 to B's column 0 needs a link line");

        assert_eq!(columns(&layout, "B"), 0);
        assert_eq!(columns(&layout, "P"), 0);
        assert_eq!(link[0].right_fork, Some(DagEdgeKind::Direct));
        assert_eq!(link[1].left_merge, Some(DagEdgeKind::Direct));
    }

    #[test]
    fn indirect_edges_stay_distinguishable_from_direct_edges() {
        let entries = vec![entry("C", &[("A", EdgeType::Indirect)]), entry("A", &[])];
        let layout = DagLayout::compute(&entries);

        let c_row = &layout.rows[0];
        assert_eq!(c_row.pad_line[0], DagVerticalCell::Indirect);
        assert_eq!(layout.rows[1].incoming, Some(DagEdgeKind::Indirect));
    }

    #[test]
    fn direct_incoming_edge_wins_when_a_target_is_also_an_indirect_ancestor() {
        let entries = vec![
            entry(
                "merge",
                &[("A", EdgeType::Indirect), ("A", EdgeType::Direct)],
            ),
            entry("A", &[]),
        ];

        let layout = DagLayout::compute(&entries);

        assert_eq!(layout.rows[1].incoming, Some(DagEdgeKind::Direct));
    }

    #[test]
    fn missing_edges_produce_a_termination_column() {
        let entries = vec![entry("A", &[("missing-parent", EdgeType::Missing)])];
        let layout = DagLayout::compute(&entries);

        let a_row = &layout.rows[0];
        assert_eq!(a_row.termination_columns, vec![0]);
        assert!(
            !layout
                .rows
                .iter()
                .any(|row| row.commit_id == "missing-parent"),
            "a missing parent must not produce its own row"
        );
    }

    #[test]
    fn omitted_synthetic_root_terminates_cleanly() {
        // The root commit is hidden from the stream; its child's edge must terminate rather than reserve a column that is never filled by a real row.
        let entries = vec![entry("only-commit", &[("hidden-root", EdgeType::Missing)])];
        let layout = DagLayout::compute(&entries);

        assert_eq!(layout.rows.len(), 1);
        assert_eq!(layout.rows[0].termination_columns, vec![0]);
    }
}
