use jayjay_core as core;
use jayjay_core::{
    BisectChange, BisectMark, BisectPhase, BisectResultKind, BisectResultView, BisectState,
    BisectVerdict,
};

#[uniffi::remote(Enum)]
pub enum BisectVerdict {
    Good,
    Bad,
    Skip,
}

#[uniffi::remote(Enum)]
pub enum BisectPhase {
    Evaluating,
    PausedRangeChanged,
    Finished,
}

#[uniffi::remote(Enum)]
pub enum BisectResultKind {
    Found,
    FoundDespiteSkips,
    Indeterminate,
    Abort,
}

#[uniffi::remote(Record)]
pub struct BisectChange {
    pub change_id: String,
    pub commit_id: String,
    pub description: String,
}

#[uniffi::remote(Record)]
pub struct BisectMark {
    pub commit_id: String,
    pub verdict: core::BisectVerdict,
}

#[uniffi::remote(Record)]
pub struct BisectResultView {
    pub kind: core::BisectResultKind,
    pub message: String,
    pub bad: Vec<core::BisectChange>,
    pub possibly_bad: Vec<core::BisectChange>,
    pub kept: Vec<core::BisectChange>,
    pub restore_note: Option<String>,
}

#[uniffi::remote(Record)]
pub struct BisectState {
    pub phase: core::BisectPhase,
    pub status: String,
    pub candidate: Option<core::BisectChange>,
    pub marks: Vec<core::BisectMark>,
    pub focused_commit_ids: Vec<String>,
    pub command_output: String,
    pub command_error: Option<String>,
    pub result: Option<core::BisectResultView>,
}
