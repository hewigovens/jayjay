use serde::Deserialize;

use super::super::checks::{self, CheckState};
use crate::types::ChecksStatus;

/// GitLab commit detail carries the combined pipeline `status` for the sha.
/// `status` is null when the commit has no pipeline.
#[derive(Deserialize)]
pub(super) struct GitLabCommitStatus {
    #[serde(default)]
    status: Option<GitLabPipelineState>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum GitLabPipelineState {
    Success,
    Failed,
    Canceled,
    Skipped,
    Created,
    Pending,
    Running,
    Preparing,
    Scheduled,
    Manual,
    WaitingForResource,
    #[serde(other)]
    Unknown,
}

impl GitLabCommitStatus {
    pub(super) fn checks(&self) -> ChecksStatus {
        let state = self
            .status
            .as_ref()
            .map(GitLabPipelineState::to_check_state);
        checks::rollup(state)
    }
}

impl GitLabPipelineState {
    fn to_check_state(&self) -> CheckState {
        match self {
            // Skipped is non-blocking; treat it as a passing/neutral outcome.
            Self::Success | Self::Skipped => CheckState::Success,
            Self::Failed | Self::Canceled => CheckState::Failure,
            _ => CheckState::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status_checks(json: &str) -> ChecksStatus {
        serde_json::from_str::<GitLabCommitStatus>(json)
            .unwrap()
            .checks()
    }

    #[test]
    fn maps_pipeline_states_to_checks() {
        assert_eq!(
            status_checks(r#"{"status": "success"}"#),
            ChecksStatus::Passing
        );
        assert_eq!(
            status_checks(r#"{"status": "running"}"#),
            ChecksStatus::Pending
        );
        assert_eq!(
            status_checks(r#"{"status": "failed"}"#),
            ChecksStatus::Failing
        );
        assert_eq!(
            status_checks(r#"{"status": "canceled"}"#),
            ChecksStatus::Failing
        );
        // No pipeline for the commit.
        assert_eq!(status_checks(r#"{"status": null}"#), ChecksStatus::None);
        assert_eq!(status_checks(r#"{}"#), ChecksStatus::None);
        // An unknown future status should not read as failing.
        assert_eq!(
            status_checks(r#"{"status": "brand_new"}"#),
            ChecksStatus::Pending
        );
    }
}
