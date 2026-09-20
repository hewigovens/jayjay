use serde::Deserialize;

use super::super::checks::CheckState;

/// A `statusCheckRollup` entry. GitHub mixes CheckRun (status + conclusion) and
/// legacy StatusContext (`state` only) shapes; we read both and let `state()` pick.
#[derive(Deserialize)]
pub(super) struct GhCheckRun {
    #[serde(default)]
    status: GhCheckStatus,
    #[serde(default)]
    conclusion: GhCheckConclusion,
    /// Present only on StatusContext entries (commit-status API).
    state: Option<GhStatusContextState>,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum GhStatusContextState {
    Success,
    Failure,
    Error,
    #[serde(other)]
    Pending,
}

#[derive(Deserialize, Default, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum GhCheckStatus {
    Completed,
    InProgress,
    Queued,
    #[default]
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Default, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum GhCheckConclusion {
    Success,
    Failure,
    Neutral,
    Cancelled,
    TimedOut,
    ActionRequired,
    #[default]
    #[serde(other)]
    Unknown,
}

impl GhCheckRun {
    pub(super) fn state(&self) -> CheckState {
        // StatusContext entries carry only `state`; map it directly.
        if let Some(state) = &self.state {
            return match state {
                GhStatusContextState::Success => CheckState::Success,
                GhStatusContextState::Failure | GhStatusContextState::Error => CheckState::Failure,
                GhStatusContextState::Pending => CheckState::Pending,
            };
        }
        if self.status != GhCheckStatus::Completed {
            CheckState::Pending
        } else if self.conclusion == GhCheckConclusion::Success {
            CheckState::Success
        } else {
            CheckState::Failure
        }
    }
}
