//! DAG layout for the jj log graph.

mod projection;
mod renderdag;
mod row_shape;

pub use row_shape::{
    DagContinuation, DagContinuationDirection, DagEdgeKind, DagLayout, DagLinkCell, DagRowShape,
    DagVerticalCell,
};
