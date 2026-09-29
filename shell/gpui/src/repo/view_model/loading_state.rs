/// An op-heads event only owes a check: jj may have written the operation we are already loaded at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingRefresh {
    CheckOperation,
    Reload,
}

/// Per-section loading flags, stale-click generation counters, and FS-watcher gates.
#[derive(Default)]
pub struct LoadingState {
    pub(crate) files: bool,
    pub diff: bool,
    pub(crate) annotate: bool,
    pub more: bool,
    pub(super) pr: bool,
    pub refresh_indicator: bool,
    /// Bumped by `select_change`; async file-load tail commits only when still current.
    pub(super) change_gen: u64,
    pub diff_gen: u64,
    pub(super) annotate_gen: u64,
    /// Bumped by `refresh_pr_info` and `select_change`; drops out-of-order PR fetches.
    pub pr_gen: u64,
    /// Bumped by `load_review_notes`; drops a reconciliation reply superseded by a newer one.
    pub(super) review_notes_gen: u64,
    /// True while any refresh/mutation runs; FS-triggered refreshes bail to avoid the snapshot-echo loop.
    pub refreshing: bool,
    /// Count of in-flight refresh/mutation tasks. `refreshing == (in_flight > 0)` keeps the gate set until all finish.
    pub in_flight: u32,
    pub operations: u32,
    /// Bumped each time `refresh()` starts; the completion discards data from a superseded run.
    pub(crate) refresh_gen: u64,
    /// An owed auto-refresh: set when an FS event arrives mid-refresh or while refreshes are suspended.
    pub pending_auto_refresh: Option<PendingRefresh>,
    pub(super) refresh_indicator_gen: u64,
    pub(super) refresh_minimum_elapsed: bool,
}
