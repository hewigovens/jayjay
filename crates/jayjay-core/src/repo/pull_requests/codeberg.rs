use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Deserialize;

use super::super::Repo;
use super::super::environment::curl_binary;
use super::super::hosted_repo::HostedRepo;
use crate::types::{ChecksStatus, PrInfo, PrState};

const CODEBERG_API_URL: &str = "https://codeberg.org/api/v1";
const URL_COMPONENT_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

#[derive(Deserialize)]
struct CodebergPrResponse {
    number: u32,
    state: PrState,
    title: String,
    #[serde(default)]
    html_url: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    merged: bool,
    base: Option<CodebergPrBranch>,
    head: Option<CodebergPrBranch>,
}

#[derive(Deserialize)]
struct CodebergPrBranch {
    #[serde(rename = "ref", default)]
    name: String,
    #[serde(default)]
    sha: String,
}

#[derive(Deserialize)]
struct CodebergCommitStatus {
    #[serde(default)]
    state: CodebergCommitStatusState,
}

#[derive(Deserialize, Default, PartialEq)]
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

pub(super) fn pr_info(repo: &Repo, remote: &HostedRepo, bookmark: &str) -> Option<PrInfo> {
    let base = repo.default_pull_request_base();
    let url = format!(
        "{}/repos/{}/{}/pulls?state=all&base={}&head={}",
        CODEBERG_API_URL,
        encode_url_component(remote.owner()),
        encode_url_component(remote.repo()),
        encode_url_component(&base),
        encode_url_component(bookmark)
    );
    let body = api_get(repo, &url)?;
    let pr = parse_prs_json(&body, &base, bookmark)?;
    let checks = pr
        .head_sha()
        .and_then(|sha| commit_status(repo, remote, sha))
        .unwrap_or(ChecksStatus::None);
    Some(pr.into_pr_info(checks))
}

fn commit_status(repo: &Repo, remote: &HostedRepo, sha: &str) -> Option<ChecksStatus> {
    let url = format!(
        "{}/repos/{}/{}/commits/{}/statuses",
        CODEBERG_API_URL,
        encode_url_component(remote.owner()),
        encode_url_component(remote.repo()),
        encode_url_component(sha)
    );
    let body = api_get(repo, &url)?;
    parse_statuses_json(&body)
}

fn api_get(repo: &Repo, url: &str) -> Option<String> {
    let output = repo
        .command_output(
            &curl_binary(),
            &["-fsSL", "--max-time", "10", url],
            "Codeberg API request",
        )
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(Repo::stdout_text(&output))
}

impl CodebergPrResponse {
    fn matches(&self, base: &str, head: &str) -> bool {
        self.base.as_ref().is_some_and(|branch| branch.name == base)
            && self.head.as_ref().is_some_and(|branch| branch.name == head)
    }

    fn head_sha(&self) -> Option<&str> {
        self.head
            .as_ref()
            .map(|branch| branch.sha.as_str())
            .filter(|sha| !sha.is_empty())
    }

    fn into_pr_info(self, checks: ChecksStatus) -> PrInfo {
        let state = if self.merged {
            PrState::Merged
        } else {
            self.state
        };
        let url = if self.html_url.is_empty() {
            self.url
        } else {
            self.html_url
        };
        PrInfo {
            number: self.number,
            state,
            title: self.title,
            url,
            checks,
        }
    }
}

fn parse_prs_json(json: &str, base: &str, head: &str) -> Option<CodebergPrResponse> {
    serde_json::from_str::<Vec<CodebergPrResponse>>(json)
        .ok()?
        .into_iter()
        .find(|pr| pr.matches(base, head))
}

fn parse_statuses_json(json: &str) -> Option<ChecksStatus> {
    let statuses: Vec<CodebergCommitStatus> = serde_json::from_str(json).ok()?;
    Some(match statuses.as_slice() {
        [] => ChecksStatus::None,
        statuses
            if statuses.iter().any(|status| {
                matches!(
                    status.state,
                    CodebergCommitStatusState::Pending | CodebergCommitStatusState::Unknown
                )
            }) =>
        {
            ChecksStatus::Pending
        }
        statuses
            if statuses.iter().any(|status| {
                matches!(
                    status.state,
                    CodebergCommitStatusState::Error
                        | CodebergCommitStatusState::Failure
                        | CodebergCommitStatusState::Warning
                )
            }) =>
        {
            ChecksStatus::Failing
        }
        _ => ChecksStatus::Passing,
    })
}

fn encode_url_component(s: &str) -> String {
    utf8_percent_encode(s, URL_COMPONENT_ENCODE_SET).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pr_for_matching_base_and_head() {
        let json = r#"[
            {
                "number": 1,
                "state": "open",
                "title": "test: verify Codeberg PR workflow",
                "html_url": "https://codeberg.org/hewig/jj-test/pulls/1",
                "merged": false,
                "base": {"ref": "main", "sha": "base-sha"},
                "head": {"ref": "feat/codeberg-pr-test", "sha": "head-sha"}
            }
        ]"#;

        let pr = parse_prs_json(json, "main", "feat/codeberg-pr-test")
            .unwrap()
            .into_pr_info(ChecksStatus::None);

        assert_eq!(pr.number, 1);
        assert_eq!(pr.state, PrState::Open);
        assert_eq!(pr.title, "test: verify Codeberg PR workflow");
        assert_eq!(pr.url, "https://codeberg.org/hewig/jj-test/pulls/1");
        assert_eq!(pr.checks, ChecksStatus::None);
    }

    #[test]
    fn parse_commit_statuses() {
        assert_eq!(
            parse_statuses_json(r#"[{"state": "success"}]"#),
            Some(ChecksStatus::Passing)
        );
        assert_eq!(
            parse_statuses_json(r#"[{"state": "pending"}]"#),
            Some(ChecksStatus::Pending)
        );
        assert_eq!(
            parse_statuses_json(r#"[{"state": "failure"}]"#),
            Some(ChecksStatus::Failing)
        );
        assert_eq!(parse_statuses_json(r#"[]"#), Some(ChecksStatus::None));
    }
}
