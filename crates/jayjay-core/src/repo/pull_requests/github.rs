use serde::Deserialize;

use super::super::Repo;
use super::super::environment::gh_binary;
use super::PrLookup;
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

/// A `statusCheckRollup` entry. GitHub mixes CheckRun (status + conclusion) and
/// legacy StatusContext (`state` only) shapes; we read both and let `state()` pick.
#[derive(Deserialize)]
struct GhCheckRun {
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

pub(super) fn pr_info(repo: &Repo, bookmark: &str) -> PrLookup {
    let Ok(output) = repo.command_output(
        &gh_binary(),
        &[
            "pr",
            "view",
            bookmark,
            "--json",
            "number,state,title,url,statusCheckRollup",
        ],
        "gh pr view",
    ) else {
        return PrLookup::Unknown;
    };
    if !output.status.success() {
        // gh exits non-zero for both "no PR" and offline/auth errors; only the former is actionable.
        return if is_no_pr_error(&Repo::stderr_text(&output)) {
            PrLookup::NotFound
        } else {
            PrLookup::Unknown
        };
    }
    match parse_pr_json(&Repo::stdout_text(&output)) {
        Some(pr) => PrLookup::Found(pr),
        None => PrLookup::Unknown,
    }
}

/// `gh pr view` reports a confirmed absence with "no pull requests found".
fn is_no_pr_error(stderr: &str) -> bool {
    stderr.contains("no pull requests found") || stderr.contains("no open pull requests found")
}

impl GhCheckRun {
    fn state(&self) -> CheckState {
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
