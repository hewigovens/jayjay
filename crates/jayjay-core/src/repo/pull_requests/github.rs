use serde::Deserialize;

use super::super::Repo;
use super::super::environment::gh_binary;
use super::checks::{self, CheckState};
use crate::types::{PrInfo, PrState};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhPrResponse {
    number: u32,
    state: GitHubPrState,
    title: String,
    url: String,
    #[serde(default)]
    status_check_rollup: Vec<GhCheckRun>,
}

/// `gh pr view` returns the state as SCREAMING_CASE.
#[derive(Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum GitHubPrState {
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

#[derive(Deserialize)]
struct GhCheckRun {
    #[serde(default)]
    status: GhCheckStatus,
    #[serde(default)]
    conclusion: GhCheckConclusion,
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

pub(super) fn pr_info(repo: &Repo, bookmark: &str) -> Option<PrInfo> {
    let output = repo
        .command_output(
            &gh_binary(),
            &[
                "pr",
                "view",
                bookmark,
                "--json",
                "number,state,title,url,statusCheckRollup",
            ],
            "gh pr view",
        )
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_pr_json(&Repo::stdout_text(&output))
}

impl GhCheckRun {
    fn state(&self) -> CheckState {
        if self.status != GhCheckStatus::Completed {
            CheckState::Pending
        } else if self.conclusion == GhCheckConclusion::Success {
            CheckState::Success
        } else {
            CheckState::Failure
        }
    }
}

fn parse_pr_json(json: &str) -> Option<PrInfo> {
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
