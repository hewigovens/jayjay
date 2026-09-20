use serde::Deserialize;

use super::super::checks;
use super::status::GhCheckRun;
use crate::types::{PrInfo, PrState};

/// `gh pr view --json` output; fields a caller did not request keep their defaults.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GhPrResponse {
    pub(crate) number: u32,
    pub(crate) state: GitHubPrState,
    pub(crate) title: String,
    pub(crate) url: String,
    #[serde(default)]
    pub(super) status_check_rollup: Vec<GhCheckRun>,
    #[serde(default)]
    pub(crate) head_ref_name: String,
    #[serde(default)]
    pub(crate) head_ref_oid: String,
    #[serde(default)]
    pub(crate) is_cross_repository: bool,
    pub(crate) head_repository_owner: Option<GhRepositoryOwner>,
    pub(crate) head_repository: Option<GhRepository>,
}

#[derive(Deserialize)]
pub(crate) struct GhRepositoryOwner {
    pub(crate) login: String,
}

#[derive(Deserialize)]
pub(crate) struct GhRepository {
    pub(crate) name: String,
}

/// `gh pr view` returns the state as SCREAMING_CASE.
#[derive(Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub(crate) enum GitHubPrState {
    Open,
    Closed,
    Merged,
}

impl From<GitHubPrState> for PrState {
    fn from(state: GitHubPrState) -> Self {
        match state {
            GitHubPrState::Open => PrState::Open,
            GitHubPrState::Closed => PrState::Closed,
            GitHubPrState::Merged => PrState::Merged,
        }
    }
}

pub(super) fn parse_pr_json(json: &str) -> Option<PrInfo> {
    let resp: GhPrResponse = serde_json::from_str(json).ok()?;
    let checks = checks::rollup(resp.status_check_rollup.iter().map(GhCheckRun::state));
    Some(PrInfo {
        number: resp.number,
        state: resp.state.into(),
        title: resp.title,
        url: resp.url,
        checks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ChecksStatus;

    #[test]
    fn parse_pr_with_passing_checks() {
        let json = r#"{
            "number": 42, "state": "OPEN", "title": "Fix the thing",
            "url": "https://github.com/o/r/pull/42",
            "statusCheckRollup": [{"name": "ci", "status": "COMPLETED", "conclusion": "SUCCESS"}]
        }"#;
        let pr = parse_pr_json(json).unwrap();
        assert_eq!(pr.number, 42);
        assert_eq!(pr.state, PrState::Open);
        assert_eq!(pr.checks, ChecksStatus::Passing);
    }

    #[test]
    fn parse_pr_with_failing_checks() {
        let json = r#"{
            "number": 7, "state": "MERGED", "title": "WIP",
            "url": "https://github.com/o/r/pull/7",
            "statusCheckRollup": [
                {"name": "ci", "status": "COMPLETED", "conclusion": "SUCCESS"},
                {"name": "lint", "status": "COMPLETED", "conclusion": "FAILURE"}
            ]
        }"#;
        let pr = parse_pr_json(json).unwrap();
        assert_eq!(pr.state, PrState::Merged);
        assert_eq!(pr.checks, ChecksStatus::Failing);
    }

    #[test]
    fn status_context_failure_does_not_read_as_pending() {
        // A StatusContext FAILURE (only `state`, no status/conclusion) must read as Failing, not Pending.
        let json = r#"{
            "number": 9, "state": "OPEN", "title": "External CI",
            "url": "https://github.com/o/r/pull/9",
            "statusCheckRollup": [
                {"__typename": "CheckRun", "name": "ci", "status": "COMPLETED", "conclusion": "SUCCESS"},
                {"__typename": "StatusContext", "context": "jenkins", "state": "FAILURE"}
            ]
        }"#;
        assert_eq!(parse_pr_json(json).unwrap().checks, ChecksStatus::Failing);
    }

    #[test]
    fn status_context_success_reads_as_passing() {
        let json = r#"{
            "number": 10, "state": "OPEN", "title": "External CI green",
            "url": "https://github.com/o/r/pull/10",
            "statusCheckRollup": [
                {"__typename": "StatusContext", "context": "jenkins", "state": "SUCCESS"}
            ]
        }"#;
        assert_eq!(parse_pr_json(json).unwrap().checks, ChecksStatus::Passing);
    }

    #[test]
    fn parse_pr_with_pending_checks() {
        let json = r#"{
            "number": 3, "state": "OPEN", "title": "In progress",
            "url": "https://github.com/o/r/pull/3",
            "statusCheckRollup": [
                {"name": "ci", "status": "COMPLETED", "conclusion": "SUCCESS"},
                {"name": "deploy", "status": "IN_PROGRESS"}
            ]
        }"#;
        assert_eq!(parse_pr_json(json).unwrap().checks, ChecksStatus::Pending);
    }
}
