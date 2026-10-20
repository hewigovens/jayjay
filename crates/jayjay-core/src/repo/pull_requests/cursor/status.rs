use serde::Deserialize;

use super::super::checks::{self, CheckState};
use crate::types::ChecksStatus;

#[derive(Deserialize)]
pub(super) struct OriginCheckRun {
    #[serde(default)]
    status: String,
    #[serde(default)]
    conclusion: Option<String>,
}

impl OriginCheckRun {
    fn state(&self) -> CheckState {
        match (self.status.as_str(), self.conclusion.as_deref()) {
            ("completed", Some("success" | "neutral" | "skipped")) => CheckState::Success,
            ("completed", _) => CheckState::Failure,
            _ => CheckState::Pending,
        }
    }

    pub(super) fn rollup(runs: &[Self]) -> ChecksStatus {
        checks::rollup(runs.iter().map(Self::state))
    }
}
