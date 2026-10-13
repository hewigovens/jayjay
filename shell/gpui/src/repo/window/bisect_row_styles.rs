use std::collections::{HashMap, HashSet};

use jayjay_core::{BisectPhase, BisectState, BisectVerdict};

#[derive(Default)]
pub(super) struct BisectRowStyles<'a> {
    focused: Option<HashSet<&'a str>>,
    candidate: Option<&'a str>,
    marks: HashMap<&'a str, BisectVerdict>,
}

pub(super) struct BisectRowStyle {
    pub dimmed: bool,
    pub candidate: bool,
    pub badge: Option<BisectVerdict>,
}

impl<'a> BisectRowStyles<'a> {
    pub fn new(state: Option<&'a BisectState>) -> Self {
        let Some(state) = state else {
            return Self::default();
        };
        let evaluating = state.phase == BisectPhase::Evaluating;
        Self {
            focused: evaluating.then(|| {
                state
                    .focused_commit_ids
                    .iter()
                    .map(String::as_str)
                    .collect()
            }),
            candidate: state
                .candidate
                .as_ref()
                .filter(|_| evaluating)
                .map(|candidate| candidate.commit_id.as_str()),
            marks: if state.phase == BisectPhase::Finished {
                HashMap::new()
            } else {
                state
                    .marks
                    .iter()
                    .map(|mark| (mark.commit_id.as_str(), mark.verdict))
                    .collect()
            },
        }
    }

    pub fn for_commit(&self, commit_id: &str) -> BisectRowStyle {
        BisectRowStyle {
            dimmed: self
                .focused
                .as_ref()
                .is_some_and(|focused| !focused.contains(commit_id)),
            candidate: self.candidate == Some(commit_id),
            badge: self.marks.get(commit_id).copied(),
        }
    }
}
