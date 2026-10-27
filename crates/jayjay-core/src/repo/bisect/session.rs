use std::sync::MutexGuard;

use jj_lib::backend::CommitId;
use jj_lib::bisect::Evaluation;
use jj_lib::commit::Commit;
use jj_lib::object_id::ObjectId as _;

use super::result::change_view;
use crate::repo::Repo;
use crate::types::*;

#[derive(Clone)]
pub(super) struct StoredMark {
    pub id: CommitId,
    pub evaluation: Evaluation,
}

#[derive(Clone)]
pub(super) enum RestoreTarget {
    Change {
        change_id: String,
        parents: Vec<CommitId>,
    },
    Parents {
        parents: Vec<CommitId>,
    },
}

#[derive(Clone)]
pub(in crate::repo) struct Session {
    pub(super) range: Vec<CommitId>,
    pub(super) good_ends: Vec<String>,
    pub(super) bad_ends: Vec<String>,
    pub(super) marks: Vec<StoredMark>,
    pub(super) candidate: Option<Commit>,
    pub(super) restore: RestoreTarget,
    pub(super) temps: Vec<String>,
    phase: BisectPhase,
    focused: Vec<String>,
    command_output: String,
    command_error: Option<String>,
    result: Option<BisectResultView>,
}

pub(super) trait ToEvaluation {
    fn to_evaluation(self) -> Evaluation;
}

impl ToEvaluation for BisectVerdict {
    fn to_evaluation(self) -> Evaluation {
        match self {
            Self::Good => Evaluation::Good,
            Self::Bad => Evaluation::Bad,
            Self::Skip => Evaluation::Skip,
        }
    }
}

impl Session {
    pub(super) fn new(
        range: Vec<CommitId>,
        good_ends: Vec<String>,
        bad_ends: Vec<String>,
        restore: RestoreTarget,
    ) -> Self {
        Self {
            range,
            good_ends,
            bad_ends,
            marks: Vec::new(),
            candidate: None,
            restore,
            temps: Vec::new(),
            phase: BisectPhase::Evaluating,
            focused: Vec::new(),
            command_output: String::new(),
            command_error: None,
            result: None,
        }
    }

    pub(super) fn phase(&self) -> BisectPhase {
        self.phase
    }

    pub(super) fn guard_candidate(&self, commit_hex: &str) -> JayResult<CommitId> {
        let Some(candidate) = &self.candidate else {
            return Err(JayError::refused("There is no candidate to mark."));
        };
        CommitId::try_from_hex(commit_hex)
            .filter(|id| id == candidate.id())
            .ok_or_else(|| {
                JayError::refused("Only the current candidate can be marked, and only once.")
            })
    }

    pub(super) fn push_mark(&mut self, id: CommitId, evaluation: Evaluation) {
        self.marks.push(StoredMark { id, evaluation });
    }

    pub(super) fn pop_mark(&mut self) {
        self.marks.pop();
    }

    pub(super) fn set_remaining(&mut self, focused: Vec<String>) {
        self.focused = focused;
    }

    pub(super) fn set_candidate(&mut self, commit: Commit) {
        self.candidate = Some(commit);
        self.phase = BisectPhase::Evaluating;
    }

    pub(super) fn finish(&mut self, result: BisectResultView) {
        self.phase = BisectPhase::Finished;
        self.candidate = None;
        self.focused.clear();
        self.result = Some(result);
    }

    pub(super) fn pause(&mut self) {
        self.phase = BisectPhase::PausedRangeChanged;
        self.focused.clear();
    }

    pub(super) fn set_command_output(&mut self, output: String) {
        self.command_output = output;
        self.command_error = None;
    }

    pub(super) fn set_command_error(&mut self, message: String) {
        self.command_error = Some(message);
    }

    pub(super) fn state(&self) -> BisectState {
        let status = match self.phase {
            BisectPhase::Evaluating => {
                let subject = self
                    .candidate
                    .as_ref()
                    .map(|change| change_subject(change.description()))
                    .unwrap_or_else(jayjay_i18n::repo::bisect_this_change);
                let count = self.focused.len();
                jayjay_i18n::repo::bisect_testing(&subject, count, steps_left(count))
            }
            BisectPhase::PausedRangeChanged => jayjay_i18n::repo::bisect_range_changed(),
            BisectPhase::Finished => self
                .result
                .as_ref()
                .map(|result| result.message.clone())
                .unwrap_or_default(),
        };
        let marks = self
            .marks
            .iter()
            .filter_map(|mark| {
                let verdict = match mark.evaluation {
                    Evaluation::Good => BisectVerdict::Good,
                    Evaluation::Bad => BisectVerdict::Bad,
                    Evaluation::Skip => BisectVerdict::Skip,
                    Evaluation::Abort => return None,
                };
                Some(BisectMark {
                    commit_id: mark.id.hex(),
                    verdict,
                })
            })
            .collect();
        BisectState {
            phase: self.phase,
            status,
            candidate: self.candidate.as_ref().map(change_view),
            marks,
            focused_commit_ids: if self.phase == BisectPhase::Evaluating {
                self.focused.clone()
            } else {
                Vec::new()
            },
            command_output: self.command_output.clone(),
            command_error: self.command_error.clone(),
            result: self.result.clone(),
        }
    }
}

fn steps_left(remaining: usize) -> usize {
    match remaining {
        0 | 1 => remaining,
        count => (count as f64).log2().ceil() as usize,
    }
}

pub(super) fn change_subject(description: &str) -> String {
    description
        .lines()
        .next()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map_or_else(jayjay_i18n::repo::bisect_this_change, str::to_owned)
}

impl Repo {
    pub(super) fn bisect_execution_lock(&self) -> MutexGuard<'_, ()> {
        self.bisect_execution
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    pub(super) fn bisect_lock(&self) -> MutexGuard<'_, Option<Session>> {
        self.bisect
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }
}
