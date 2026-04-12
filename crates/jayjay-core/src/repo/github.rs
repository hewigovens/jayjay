use serde::Deserialize;

use super::environment::gh_binary;
use super::Repo;
use crate::types::PrInfo;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhPrResponse {
    number: u32,
    state: String,
    title: String,
    url: String,
    #[serde(default)]
    status_check_rollup: Vec<GhCheckRun>,
}

#[derive(Deserialize)]
struct GhCheckRun {
    #[serde(default)]
    status: String,
    #[serde(default)]
    conclusion: String,
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
    let checks_passed = if resp.status_check_rollup.is_empty() {
        None
    } else if resp.status_check_rollup.iter().any(|c| c.status != "COMPLETED") {
        None
    } else {
        Some(resp.status_check_rollup.iter().all(|c| c.conclusion == "SUCCESS"))
    };
    Some(PrInfo {
        number: resp.number,
        state: resp.state,
        title: resp.title,
        url: resp.url,
        checks_passed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pr_with_passing_checks() {
        let json = r#"{
            "number": 42,
            "state": "OPEN",
            "title": "Fix the thing",
            "url": "https://github.com/owner/repo/pull/42",
            "statusCheckRollup": [
                {"name": "ci", "status": "COMPLETED", "conclusion": "SUCCESS"}
            ]
        }"#;
        let pr = parse_gh_pr_json(json).unwrap();
        assert_eq!(pr.number, 42);
        assert_eq!(pr.state, "OPEN");
        assert_eq!(pr.checks_passed, Some(true));
    }

    #[test]
    fn parse_pr_with_failing_checks() {
        let json = r#"{
            "number": 7,
            "state": "OPEN",
            "title": "WIP",
            "url": "https://github.com/o/r/pull/7",
            "statusCheckRollup": [
                {"name": "ci", "status": "COMPLETED", "conclusion": "SUCCESS"},
                {"name": "lint", "status": "COMPLETED", "conclusion": "FAILURE"}
            ]
        }"#;
        let pr = parse_gh_pr_json(json).unwrap();
        assert_eq!(pr.checks_passed, Some(false));
    }

    #[test]
    fn parse_pr_with_pending_checks() {
        let json = r#"{
            "number": 3,
            "state": "OPEN",
            "title": "In progress",
            "url": "https://github.com/o/r/pull/3",
            "statusCheckRollup": [
                {"name": "ci", "status": "COMPLETED", "conclusion": "SUCCESS"},
                {"name": "deploy", "status": "IN_PROGRESS"}
            ]
        }"#;
        let pr = parse_gh_pr_json(json).unwrap();
        assert_eq!(pr.checks_passed, None);
    }

    #[test]
    fn parse_pr_with_no_checks() {
        let json = r#"{
            "number": 1,
            "state": "MERGED",
            "title": "Ship it",
            "url": "https://github.com/o/r/pull/1",
            "statusCheckRollup": []
        }"#;
        let pr = parse_gh_pr_json(json).unwrap();
        assert_eq!(pr.checks_passed, None);
    }

    #[test]
    fn parse_pr_without_check_field() {
        let json = r#"{
            "number": 1,
            "state": "OPEN",
            "title": "No checks",
            "url": "https://github.com/o/r/pull/1"
        }"#;
        let pr = parse_gh_pr_json(json).unwrap();
        assert_eq!(pr.checks_passed, None);
    }
}
