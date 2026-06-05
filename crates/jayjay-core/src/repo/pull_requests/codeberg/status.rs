use serde::Deserialize;

use super::super::checks::{self, CheckState};
use crate::types::ChecksStatus;

/// Forgejo combined status: pre-rolled-up `state` across the latest per context.
#[derive(Deserialize)]
pub(super) struct CodebergCombinedStatus {
    #[serde(default)]
    state: CodebergCommitStatusState,
    #[serde(default)]
    total_count: u32,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "lowercase")]
enum CodebergCommitStatusState {
    Pending,
    Success,
    Error,
    Failure,
    Warning,
    #[default]
    #[serde(other)]
    Unknown,
}

impl CodebergCombinedStatus {
    // Pre-aggregated into one state; feed 0-or-1 checks through the shared rollup.
    pub(super) fn checks(&self) -> ChecksStatus {
        let state = (self.total_count > 0).then(|| self.state.to_check_state());
        checks::rollup(state)
    }
}

impl CodebergCommitStatusState {
    fn to_check_state(&self) -> CheckState {
        match self {
            Self::Success => CheckState::Success,
            Self::Pending | Self::Unknown => CheckState::Pending,
            Self::Error | Self::Failure | Self::Warning => CheckState::Failure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combined_status_rolls_up_latest_state() {
        // Historical "pending" entries don't stick; the combined state rolls up.
        fn status_checks(json: &str) -> ChecksStatus {
            serde_json::from_str::<CodebergCombinedStatus>(json)
                .unwrap()
                .checks()
        }

        assert_eq!(
            status_checks(r#"{"state": "success", "total_count": 2}"#),
            ChecksStatus::Passing
        );
        assert_eq!(
            status_checks(r#"{"state": "pending", "total_count": 1}"#),
            ChecksStatus::Pending
        );
        assert_eq!(
            status_checks(r#"{"state": "failure", "total_count": 1}"#),
            ChecksStatus::Failing
        );
        assert_eq!(
            status_checks(r#"{"state": "", "total_count": 0}"#),
            ChecksStatus::None
        );
    }
}
