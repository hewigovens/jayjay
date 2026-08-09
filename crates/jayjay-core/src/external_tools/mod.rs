mod apply;
mod content;
mod diff;
mod invocation;
mod merge;
mod output;
mod scan;

pub use apply::apply_external_diff_selections;
pub use diff::{
    ExternalDiffFile, ExternalDiffSelection, ExternalDiffSide, diff_edit_ranges, load_external_diff,
};
pub use invocation::{ExternalToolInvocation, parse_external_tool_invocation};
pub use merge::{
    ExternalMerge, ExternalMergeResolution, conflict_marker_count, has_conflict_marker_remnants,
    load_external_merge, save_external_merge,
};

pub const JJ_INSTRUCTIONS: &str = "JJ-INSTRUCTIONS";
