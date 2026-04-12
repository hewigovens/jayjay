use serde::Deserialize;

use super::environment::gh_binary;
use super::Repo;
use crate::types::{ChecksStatus, PrInfo, PrState};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhPrResponse {
    number: u32,
    state: PrState,
    title: String,
    url: String,
    #[serde(default)]
    status_check_rollup: Vec<GhCheckRun>,
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

impl Repo {
    /// Query `gh pr view` for a bookmark. Returns None if gh is missing, unauthenticated, or no PR exists.
    pub fn gh_pr_info(&self, bookmark: &str) -> Option<PrInfo> {
        if bookmark.is_empty() {
            return None;
        }
        let output = self
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
        parse_gh_pr_json(&Self::stdout_text(&output))
    }
}

fn parse_gh_pr_json(json: &str) -> Option<PrInfo> {
    let resp: GhPrResponse = serde_json::from_str(json).ok()?;
    let checks = match resp.status_check_rollup.as_slice() {
        [] => ChecksStatus::None,
        runs if runs.iter().any(|c| c.status != GhCheckStatus::Completed) => ChecksStatus::Pending,
        runs if runs.iter().all(|c| c.conclusion == GhCheckConclusion::Success) => {
            ChecksStatus::Passing
        }
        _ => ChecksStatus::Failing,
    };
    Some(PrInfo {
        number: resp.number,
        state: resp.state,
        title: resp.title,
        url: resp.url,
        checks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pr_with_passing_checks() {
        let json = r#"{
            "number": 42, "state": "OPEN", "title": "Fix the thing",
            "url": "https://github.com/o/r/pull/42",
            "statusCheckRollup": [{"name": "ci", "status": "COMPLETED", "conclusion": "SUCCESS"}]
        }"#;
        let pr = parse_gh_pr_json(json).unwrap();
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
        let pr = parse_gh_pr_json(json).unwrap();
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
        assert_eq!(parse_gh_pr_json(json).unwrap().checks, ChecksStatus::Pending);
    }
}
