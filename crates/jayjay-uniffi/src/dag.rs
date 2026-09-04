use jayjay_core as core;
use jayjay_core::LogGraphPage;
use jayjay_core::dag::{
    DagContinuation, DagContinuationDirection, DagEdgeKind, DagLayout, DagLinkCell, DagRowShape,
    DagVerticalCell,
};

#[uniffi::remote(Record)]
pub struct DagLayout {
    pub rows: Vec<core::dag::DagRowShape>,
    pub logical_column_count: u32,
}

#[uniffi::remote(Record)]
pub struct LogGraphPage {
    pub entries: Vec<core::GraphEntry>,
    pub layout: core::dag::DagLayout,
    pub has_more: bool,
    pub applied_limit: u32,
}

#[uniffi::remote(Record)]
pub struct DagRowShape {
    pub commit_id: String,
    pub node_column: u32,
    pub incoming: Option<core::dag::DagEdgeKind>,
    pub node_line: Vec<core::dag::DagVerticalCell>,
    pub link_line: Option<Vec<core::dag::DagLinkCell>>,
    pub termination_columns: Vec<u32>,
    pub pad_line: Vec<core::dag::DagVerticalCell>,
    pub continuations: Vec<core::dag::DagContinuation>,
    pub elided_fork_column: Option<u32>,
}

#[uniffi::remote(Record)]
pub struct DagContinuation {
    pub key: String,
    pub edge_kind: core::dag::DagEdgeKind,
    pub direction: core::dag::DagContinuationDirection,
    pub related_commit_id: String,
}

#[uniffi::remote(Enum)]
pub enum DagContinuationDirection {
    Outgoing,
    Incoming,
}

#[uniffi::remote(Enum)]
pub enum DagVerticalCell {
    Empty,
    Direct,
    Indirect,
}

#[uniffi::remote(Record)]
pub struct DagLinkCell {
    pub vertical: Option<core::dag::DagEdgeKind>,
    pub horizontal: Option<core::dag::DagEdgeKind>,
    pub left_fork: Option<core::dag::DagEdgeKind>,
    pub right_fork: Option<core::dag::DagEdgeKind>,
    pub left_merge: Option<core::dag::DagEdgeKind>,
    pub right_merge: Option<core::dag::DagEdgeKind>,
    pub is_child: bool,
}

#[uniffi::remote(Enum)]
pub enum DagEdgeKind {
    Direct,
    Indirect,
}

#[uniffi::export]
fn compute_dag_layout(entries: Vec<core::GraphEntry>) -> core::dag::DagLayout {
    core::dag::DagLayout::compute(&entries)
}

#[cfg(test)]
mod tests {
    use jayjay_core::{
        ChangeInfo, CommitAuthor, EdgeType, GraphEdge, GraphEntry, NewChangeEligibility, ShortId,
    };

    use jayjay_core::dag::DagVerticalCell;

    use super::compute_dag_layout;

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

    /// Exercises the exported `compute_dag_layout` boundary function — the same function SwiftUI and GPUI call — over a row with a fork, a merge, an indirect edge, and a missing-edge termination, proving every field survives the FFI-facing conversion.
    #[test]
    fn compute_dag_layout_round_trips_fork_merge_indirect_and_termination() {
        let entries = vec![
            entry(
                "merge",
                &[
                    ("direct-parent", EdgeType::Direct),
                    ("indirect-parent", EdgeType::Indirect),
                    ("missing-parent", EdgeType::Missing),
                    ("outside-parent", EdgeType::Direct),
                ],
            ),
            entry("direct-parent", &[]),
            entry("indirect-parent", &[]),
        ];

        let layout = compute_dag_layout(entries);

        assert_eq!(layout.rows.len(), 3);
        assert_eq!(layout.logical_column_count, 4);

        let merge_row = &layout.rows[0];
        assert_eq!(merge_row.commit_id, "merge");
        assert_eq!(merge_row.node_column, 0);
        // The direct first parent keeps the node column, the missing parent terminates in its lane, and the off-page parent forks aside.
        assert_eq!(merge_row.pad_line[0], DagVerticalCell::Direct);
        assert_eq!(merge_row.termination_columns, vec![2]);
        assert!(merge_row.elided_fork_column.is_some_and(|fork| fork > 0));
        assert_eq!(merge_row.continuations.len(), 1);
        assert_eq!(
            merge_row.continuations[0].direction,
            jayjay_core::dag::DagContinuationDirection::Outgoing
        );
        assert_eq!(
            merge_row.continuations[0].related_commit_id,
            "outside-parent"
        );

        let link_line = merge_row
            .link_line
            .as_ref()
            .expect("fork/merge row needs a link line");
        assert!(
            link_line.iter().any(|cell| {
                cell.left_fork.is_some()
                    || cell.right_fork.is_some()
                    || cell.left_merge.is_some()
                    || cell.right_merge.is_some()
            }),
            "expected at least one fork or merge segment linking the parents: {link_line:?}"
        );
        assert!(
            link_line.iter().any(|cell| cell.is_child),
            "expected one cell to mark the merge node as the link line's child column"
        );

        let direct_row = &layout.rows[1];
        assert_eq!(direct_row.node_column, 0);
        assert_eq!(
            direct_row.incoming,
            Some(jayjay_core::dag::DagEdgeKind::Direct)
        );
        let indirect_row = &layout.rows[2];
        assert_eq!(indirect_row.node_column, 1);
        assert_eq!(
            indirect_row.incoming,
            Some(jayjay_core::dag::DagEdgeKind::Indirect)
        );
        assert!(
            merge_row.pad_line[1] == DagVerticalCell::Indirect
                || merge_row.node_line[1] == DagVerticalCell::Indirect,
            "the indirect edge should stay visually distinguishable from the direct one"
        );
    }
}
