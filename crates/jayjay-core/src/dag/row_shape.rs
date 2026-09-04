//! App-owned structural representation of a rendered DAG row.
//!
//! Mirrors the shape of `sapling_renderdag::GraphRow` without exposing any upstream type, so the pre-1.0 dependency stays fully behind `renderdag`.

/// Whether an edge is an immediate parent or a synthesized link to a further ancestor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DagEdgeKind {
    Direct,
    Indirect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DagContinuationDirection {
    Outgoing,
    Incoming,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DagContinuation {
    pub key: String,
    pub edge_kind: DagEdgeKind,
    pub direction: DagContinuationDirection,
    pub related_commit_id: String,
}

/// A cell in a row's node line or pad line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DagVerticalCell {
    Empty,
    Direct,
    Indirect,
}

/// A cell in a row's link line, describing every edge segment that can pass through it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DagLinkCell {
    pub vertical: Option<DagEdgeKind>,
    pub horizontal: Option<DagEdgeKind>,
    pub left_fork: Option<DagEdgeKind>,
    pub right_fork: Option<DagEdgeKind>,
    pub left_merge: Option<DagEdgeKind>,
    pub right_merge: Option<DagEdgeKind>,
    /// True when the node that owns this link row is the child column for this cell.
    pub is_child: bool,
}

/// The structural shape of one rendered row: the node's column and every band around it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DagRowShape {
    pub commit_id: String,
    pub node_column: u32,
    pub incoming: Option<DagEdgeKind>,
    pub node_line: Vec<DagVerticalCell>,
    pub link_line: Option<Vec<DagLinkCell>>,
    pub termination_columns: Vec<u32>,
    pub pad_line: Vec<DagVerticalCell>,
    pub continuations: Vec<DagContinuation>,
    /// Lane for an off-page parent's elided fork stub, set only when the node column still carries a surviving first-parent edge.
    pub elided_fork_column: Option<u32>,
}

/// A full graph, ordered top to bottom, plus the logical column count needed to draw it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DagLayout {
    pub rows: Vec<DagRowShape>,
    pub logical_column_count: u32,
}

impl DagLayout {
    pub fn row(&self, commit_id: &str) -> Option<&DagRowShape> {
        self.rows.iter().find(|row| row.commit_id == commit_id)
    }
}
