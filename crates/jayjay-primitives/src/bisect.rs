#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BisectVerdict {
    Good,
    Bad,
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BisectPhase {
    Evaluating,
    PausedRangeChanged,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BisectResultKind {
    Found,
    FoundDespiteSkips,
    Indeterminate,
    Abort,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BisectChange {
    pub change_id: String,
    pub commit_id: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BisectMark {
    pub commit_id: String,
    pub verdict: BisectVerdict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BisectResultView {
    pub kind: BisectResultKind,
    pub message: String,
    pub bad: Vec<BisectChange>,
    pub possibly_bad: Vec<BisectChange>,
    pub kept: Vec<BisectChange>,
    pub restore_note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BisectState {
    pub phase: BisectPhase,
    pub status: String,
    pub candidate: Option<BisectChange>,
    pub marks: Vec<BisectMark>,
    pub focused_commit_ids: Vec<String>,
    pub command_output: String,
    pub command_error: Option<String>,
    pub result: Option<BisectResultView>,
}
