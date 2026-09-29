/// An op-heads event only owes a check: jj may have written the operation we are already loaded at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingRefresh {
    CheckOperation,
    Reload,
}

#[derive(Default)]
pub struct LoadingState {
    pub(crate) files: bool,
    pub diff: bool,
    pub(crate) annotate: bool,
    pub more: bool,
    pub(super) pr: bool,
    pub refresh_indicator: bool,
    pub(super) change_gen: u64,
    pub diff_gen: u64,
    pub(super) annotate_gen: u64,
    pub pr_gen: u64,
    pub(super) review_notes_gen: u64,
    /// True while any refresh/mutation runs; FS-triggered refreshes bail to avoid the snapshot-echo loop.
    pub refreshing: bool,
    /// `refreshing == (in_flight > 0)` keeps the gate set until all finish.
    pub in_flight: u32,
    pub operations: u32,
    pub(crate) refresh_gen: u64,
    /// An owed auto-refresh: set when an FS event arrives mid-refresh or while refreshes are suspended.
    pub pending_auto_refresh: Option<PendingRefresh>,
    pub(super) refresh_indicator_gen: u64,
    pub(super) refresh_minimum_elapsed: bool,
}
