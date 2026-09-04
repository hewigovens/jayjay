//! Adapter from the `sapling-renderdag` crate's `GraphRowRenderer` to JayJay's app-owned row shapes.
//!
//! No upstream type crosses this module boundary.

use std::collections::{HashMap, HashSet};

use renderdag::{Ancestor, GraphRow, GraphRowRenderer, LinkLine, NodeLine, PadLine, Renderer};

use super::projection::EdgeId;
use super::row_shape::{
    DagContinuation, DagContinuationDirection, DagEdgeKind, DagLayout, DagLinkCell, DagRowShape,
    DagVerticalCell,
};
use crate::types::{EdgeType, GraphEntry};

pub(super) fn render(
    entries: &[GraphEntry],
    cut_edges: &std::collections::HashSet<EdgeId>,
    continuations: &[Vec<DagContinuation>],
) -> DagLayout {
    let mut renderer = GraphRowRenderer::<String>::new();
    let mut rows = Vec::with_capacity(entries.len());
    let mut logical_column_count = 0;
    let mut incoming = HashMap::new();
    let visible_commit_ids = entries
        .iter()
        .map(|entry| entry.change.commit_id.id.as_str())
        .collect::<HashSet<_>>();

    for (source_index, entry) in entries.iter().enumerate() {
        let commit_id = entry.change.commit_id.id.clone();
        let parents = entry
            .edges
            .iter()
            .enumerate()
            .filter_map(|(edge_index, edge)| {
                let cut = cut_edges.contains(&EdgeId {
                    source_index,
                    edge_index,
                });
                (visible_commit_ids.contains(edge.target.as_str()) || !cut)
                    .then(|| to_ancestor(edge, cut))
            })
            .collect();
        let row = renderer.next_row(commit_id.clone(), parents, String::new(), String::new());
        debug_assert_eq!(
            row.node, commit_id,
            "renderer emitted a row for a different commit than requested"
        );
        let shape = to_row_shape(
            row,
            incoming.remove(&commit_id),
            continuations[source_index].clone(),
        );
        logical_column_count = logical_column_count
            .max(shape.node_line.len() as u32)
            .max(shape.pad_line.len() as u32);
        rows.push(shape);
        register_incoming_edges(&mut incoming, entry, source_index, cut_edges);
    }

    DagLayout {
        rows,
        logical_column_count,
    }
}

fn register_incoming_edges(
    incoming: &mut HashMap<String, DagEdgeKind>,
    entry: &GraphEntry,
    source_index: usize,
    cut_edges: &std::collections::HashSet<EdgeId>,
) {
    for (edge_index, edge) in entry.edges.iter().enumerate() {
        if cut_edges.contains(&EdgeId {
            source_index,
            edge_index,
        }) {
            continue;
        }
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

fn to_ancestor(edge: &crate::types::GraphEdge, cut: bool) -> Ancestor<String> {
    if cut {
        return Ancestor::Anonymous;
    }
    match edge.edge_type {
        EdgeType::Direct => Ancestor::Parent(edge.target.clone()),
        EdgeType::Indirect => Ancestor::Ancestor(edge.target.clone()),
        EdgeType::Missing => Ancestor::Anonymous,
    }
}

fn to_row_shape(
    row: GraphRow<String>,
    incoming: Option<DagEdgeKind>,
    continuations: Vec<DagContinuation>,
) -> DagRowShape {
    let node_column = node_column(&row.node_line);
    let node_line: Vec<DagVerticalCell> = row.node_line.iter().map(to_vertical_cell).collect();
    let mut pad_line = row
        .pad_lines
        .iter()
        .map(pad_to_vertical_cell)
        .collect::<Vec<_>>();
    let link_line = row
        .link_line
        .map(|cells| cells.iter().map(to_link_cell).collect());
    let mut termination_columns: Vec<u32> = row
        .term_line
        .map(|flags| {
            flags
                .iter()
                .enumerate()
                .filter_map(|(column, &terminates)| terminates.then_some(column as u32))
                .collect()
        })
        .unwrap_or_default();
    let mut elided_fork_column = None;
    let has_outgoing = continuations
        .iter()
        .any(|continuation| continuation.direction == DagContinuationDirection::Outgoing);
    if has_outgoing {
        let node_survives = matches!(
            pad_line.get(node_column as usize),
            Some(DagVerticalCell::Direct | DagVerticalCell::Indirect)
        );
        if node_survives {
            let width = node_line.len().max(pad_line.len()).max(
                termination_columns
                    .iter()
                    .copied()
                    .max()
                    .map_or(0, |column| column as usize + 1),
            );
            while pad_line.len() <= width {
                pad_line.push(DagVerticalCell::Empty);
            }
            elided_fork_column = Some(width as u32);
        } else {
            if let Some(cell) = pad_line.get_mut(node_column as usize) {
                *cell = DagVerticalCell::Empty;
            }
            if !termination_columns.contains(&node_column) {
                termination_columns.push(node_column);
                termination_columns.sort_unstable();
            }
        }
    }

    DagRowShape {
        commit_id: row.node,
        node_column,
        incoming,
        node_line,
        link_line,
        termination_columns,
        pad_line,
        continuations,
        elided_fork_column,
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

    fn continuation_keys(
        layout: &DagLayout,
        commit_id: &str,
        direction: DagContinuationDirection,
    ) -> Vec<String> {
        layout
            .row(commit_id)
            .unwrap_or_else(|| panic!("no row for {commit_id}"))
            .continuations
            .iter()
            .filter(|continuation| continuation.direction == direction)
            .map(|continuation| continuation.key.clone())
            .collect()
    }

    #[test]
    fn linear_history_stays_in_one_column() {
        let entries = vec![direct("C", &["B"]), direct("B", &["A"]), direct("A", &[])];
        let layout = DagLayout::compute(&entries);

        assert_eq!(layout.logical_column_count, 1);
        assert_eq!(columns(&layout, "C"), 0);
        assert_eq!(columns(&layout, "B"), 0);
        assert_eq!(columns(&layout, "A"), 0);
        assert!(layout.rows.iter().all(|row| row.continuations.is_empty()));
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
        assert!(layout.rows.iter().all(|row| row.continuations.is_empty()));
    }

    #[test]
    fn merge_with_surviving_first_parent_forks_the_elided_parent_aside() {
        let entries = vec![
            entry(
                "merge",
                &[("parent", EdgeType::Direct), ("off-page", EdgeType::Direct)],
            ),
            direct("parent", &[]),
        ];

        let layout = DagLayout::compute(&entries);
        let row = layout.row("merge").expect("merge row");

        assert_eq!(
            row.pad_line[row.node_column as usize],
            DagVerticalCell::Direct
        );
        assert!(!row.termination_columns.contains(&row.node_column));
        let fork = row.elided_fork_column.expect("elided parent forks aside");
        assert!(fork > row.node_column);
        assert_eq!(row.continuations.len(), 1);
    }

    #[test]
    fn parent_outside_the_page_becomes_one_outgoing_continuation() {
        let entries = vec![direct("C", &["outside-parent"])];

        let layout = DagLayout::compute(&entries);

        let row = &layout.rows[0];
        assert_eq!(row.termination_columns, vec![row.node_column]);
        assert_eq!(
            row.pad_line[row.node_column as usize],
            DagVerticalCell::Empty
        );
        assert_eq!(row.continuations.len(), 1);
        assert_eq!(row.continuations[0].edge_kind, DagEdgeKind::Direct);
    }

    #[test]
    fn merge_parents_outside_the_page_become_outgoing_continuations() {
        let entries = vec![direct(
            "merge",
            &["outside-first-parent", "outside-second-parent"],
        )];

        let layout = DagLayout::compute(&entries);

        assert_eq!(layout.rows[0].continuations.len(), 2);
    }

    #[test]
    fn long_in_page_edge_becomes_paired_continuations() {
        let mut entries = vec![direct("head", &[]), direct("long-source", &["target"])];
        entries.extend((0..12).map(|index| direct(&format!("filler-{index}"), &[])));
        entries.push(direct("target", &[]));

        let layout = DagLayout::compute(&entries);

        let outgoing =
            continuation_keys(&layout, "long-source", DagContinuationDirection::Outgoing);
        let incoming = continuation_keys(&layout, "target", DagContinuationDirection::Incoming);
        assert_eq!(outgoing.len(), 1);
        assert_eq!(incoming, outgoing);
    }

    #[test]
    fn wide_graph_is_reduced_to_the_lane_budget_without_hiding_rows() {
        let mut entries = (0..9)
            .map(|index| direct(&format!("source-{index}"), &[&format!("target-{index}")]))
            .collect::<Vec<_>>();
        entries.extend((0..9).map(|index| direct(&format!("target-{index}"), &[])));

        let layout = DagLayout::compute(&entries);

        assert_eq!(layout.rows.len(), entries.len());
        assert!(layout.logical_column_count <= crate::dag::projection::MAX_VISIBLE_DAG_LANES);
        assert!(layout.rows.iter().any(|row| !row.continuations.is_empty()));
    }

    #[test]
    fn working_copy_first_parent_spine_is_never_cut() {
        let mut entries = vec![direct("working-copy", &["parent"])];
        entries[0].change.is_working_copy = true;
        entries.extend((0..13).map(|index| direct(&format!("filler-{index}"), &[])));
        entries.push(direct("parent", &[]));

        let layout = DagLayout::compute(&entries);

        assert!(layout.rows[0].continuations.is_empty());
        assert_eq!(
            layout.row("parent").expect("parent row").incoming,
            Some(DagEdgeKind::Direct)
        );
    }

    #[test]
    fn a_later_direct_edge_is_not_promoted_to_first_parent() {
        let mut entries = vec![entry(
            "working-copy",
            &[
                ("indirect-first-parent", EdgeType::Indirect),
                ("direct-second-parent", EdgeType::Direct),
            ],
        )];
        entries[0].change.is_working_copy = true;
        entries.extend((0..13).map(|index| direct(&format!("filler-{index}"), &[])));
        entries.push(direct("indirect-first-parent", &[]));
        entries.push(direct("direct-second-parent", &[]));

        let layout = DagLayout::compute(&entries);

        assert!(
            layout.rows[0]
                .continuations
                .iter()
                .any(|continuation| continuation.related_commit_id == "direct-second-parent")
        );
    }

    #[test]
    fn adjacent_first_parent_edge_survives_an_unavoidable_wide_row() {
        let parents = (0..9).map(|index| format!("p{index}")).collect::<Vec<_>>();
        let parent_refs = parents.iter().map(String::as_str).collect::<Vec<_>>();
        let mut entries = vec![direct("head", &[]), direct("merge", &parent_refs)];
        entries.extend(parents.iter().map(|parent| direct(parent, &[])));

        let layout = DagLayout::compute(&entries);

        assert!(layout.logical_column_count > crate::dag::projection::MAX_VISIBLE_DAG_LANES);
        assert!(
            layout
                .row("merge")
                .expect("merge row")
                .continuations
                .iter()
                .all(|continuation| continuation.related_commit_id != "p0")
        );
        assert_eq!(
            layout.row("p0").expect("first parent row").incoming,
            Some(DagEdgeKind::Direct)
        );
    }

    #[test]
    fn projection_is_deterministic_and_preserves_semantic_edges() {
        let mut entries = (0..9)
            .map(|index| direct(&format!("source-{index}"), &[&format!("target-{index}")]))
            .collect::<Vec<_>>();
        entries.extend((0..9).map(|index| direct(&format!("target-{index}"), &[])));
        let original_edges = entries
            .iter()
            .map(|entry| {
                entry
                    .edges
                    .iter()
                    .map(|edge| (edge.target.clone(), edge.edge_type))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let first = DagLayout::compute(&entries);
        let second = DagLayout::compute(&entries);

        assert_eq!(first, second);
        assert_eq!(
            entries
                .iter()
                .map(|entry| {
                    entry
                        .edges
                        .iter()
                        .map(|edge| (edge.target.clone(), edge.edge_type))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            original_edges
        );
    }

    #[test]
    fn continuation_kinds_and_missing_edges_remain_distinguishable() {
        let entries = vec![entry(
            "source",
            &[
                ("direct-outside", EdgeType::Direct),
                ("indirect-outside", EdgeType::Indirect),
                ("missing", EdgeType::Missing),
            ],
        )];

        let layout = DagLayout::compute(&entries);
        let continuations = &layout.rows[0].continuations;

        assert_eq!(continuations.len(), 2);
        assert!(
            continuations
                .iter()
                .any(|marker| marker.edge_kind == DagEdgeKind::Direct)
        );
        assert!(
            continuations
                .iter()
                .any(|marker| marker.edge_kind == DagEdgeKind::Indirect)
        );
        assert_eq!(layout.rows[0].termination_columns.len(), 1);
    }

    #[test]
    fn projected_connectors_share_one_dotted_termination() {
        let targets = (0..8)
            .map(|index| format!("target-{index}"))
            .collect::<Vec<_>>();
        let edges = targets
            .iter()
            .enumerate()
            .map(|(index, target)| {
                (
                    target.as_str(),
                    if index == 0 {
                        EdgeType::Direct
                    } else {
                        EdgeType::Indirect
                    },
                )
            })
            .collect::<Vec<_>>();
        let layout = DagLayout::compute(&[entry("source", &edges)]);
        let row = &layout.rows[0];

        assert_eq!(row.continuations.len(), targets.len());
        assert_eq!(row.termination_columns, vec![row.node_column]);
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
    fn terminal_octopus_merge_projects_out_of_page_parents() {
        let layout = DagLayout::compute(&[direct("merge", &["p0", "p1", "p2", "p3", "p4", "p5"])]);
        let row = &layout.rows[0];

        assert_eq!(row.continuations.len(), 6);
        assert_eq!(row.termination_columns, vec![row.node_column]);
        assert_eq!(layout.logical_column_count, 1);
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
