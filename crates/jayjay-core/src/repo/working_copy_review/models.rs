use jayjay_primitives::{
    ChangeInfo, DiffHunk, NoteSide, ReviewFileRollup, ReviewGroupState, ReviewMarkSource,
};
use jj_diff::ReviewFileSnapshot;

pub struct WorkingCopyReviewStatus {
    pub change: ChangeInfo,
    pub files: Vec<WorkingCopyReviewFile>,
}

pub struct WorkingCopyReviewFile {
    pub diff: DiffHunk,
    pub rollup: ReviewFileRollup,
    pub source: ReviewMarkSource,
    pub snapshot: ReviewFileSnapshot,
    pub display_groups: Vec<Vec<u32>>,
    pub group_states: Vec<ReviewGroupState>,
}

pub struct WorkingCopyReviewNote {
    pub expected_commit: String,
    pub path: String,
    pub side: NoteSide,
    pub line: u32,
    pub body: String,
    pub ignore_whitespace: bool,
}

/// Display-group indices are scoped to this exact commit, path, and whitespace mode; None addresses the whole file.
pub struct WorkingCopyReviewMark {
    pub expected_commit: String,
    pub path: String,
    pub display_group: Option<u32>,
    pub ignore_whitespace: bool,
    pub reviewed: bool,
}
