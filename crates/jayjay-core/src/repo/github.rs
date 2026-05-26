use serde::Deserialize;

use super::Repo;
use super::environment::gh_binary;
use crate::types::{ChecksStatus, PrInfo, PrState};

const GITHUB_HOST: &str = "github.com";

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

    /// Existing PR URL for `bookmark` if one exists, else a GitHub compose URL. None for non-github.com remotes.
    pub fn gh_pr_open_url(&self, bookmark: &str) -> Option<String> {
        if bookmark.is_empty() {
            return None;
        }
        if let Some(pr) = self.gh_pr_info(bookmark) {
            return Some(pr.url);
        }
        let remote = self.git_remote_url().ok()?;
        let slug = github_slug(&remote)?;
        let encoded = encode_path_segment(bookmark);
        Some(format!("https://{GITHUB_HOST}/{slug}/pull/new/{encoded}"))
    }
}

/// Extract `owner/repo` from a github.com remote URL (scp-ssh, ssh://, or http(s)://). Rejects other hosts.
fn github_slug(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let (host, path) = if let Some(rest) = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
    {
        rest.split_once('/')?
    } else if let Some(rest) = trimmed.strip_prefix("ssh://") {
        rest.split_once('/')?
    } else if let Some((_, rest)) = trimmed.split_once('@') {
        rest.split_once(':')?
    } else {
        return None;
    };
    // Strip optional userinfo (TOKEN@host or user:pass@host) and port.
    let host = host.rsplit_once('@').map(|(_, h)| h).unwrap_or(host);
    let host = host.split(':').next().unwrap_or(host);
    if host != GITHUB_HOST {
        return None;
    }
    let mut parts = path.trim_matches('/').split('/').filter(|s| !s.is_empty());
    let owner = parts.next()?;
    let repo = parts.next()?;
    let repo = repo.strip_suffix(".git").unwrap_or(repo);
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some(format!("{owner}/{repo}"))
}

/// Percent-encode bytes that aren't safe in a URL path segment. Preserves `/` so `feat/foo` round-trips.
fn encode_path_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn parse_gh_pr_json(json: &str) -> Option<PrInfo> {
    let resp: GhPrResponse = serde_json::from_str(json).ok()?;
    let checks = match resp.status_check_rollup.as_slice() {
        [] => ChecksStatus::None,
        runs if runs.iter().any(|c| c.status != GhCheckStatus::Completed) => ChecksStatus::Pending,
        runs if runs
            .iter()
            .all(|c| c.conclusion == GhCheckConclusion::Success) =>
        {
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
    fn slug_extracts_from_common_remote_forms() {
        let expected = Some("hewigovens/jayjay");
        assert_eq!(
            github_slug("git@github.com:hewigovens/jayjay.git").as_deref(),
            expected
        );
        assert_eq!(
            github_slug("https://github.com/hewigovens/jayjay.git\n").as_deref(),
            expected
        );
        assert_eq!(
            github_slug("ssh://git@github.com/hewigovens/jayjay.git").as_deref(),
            expected
        );
        // HTTPS with token/userinfo (e.g., from `gh auth setup-git`).
        assert_eq!(
            github_slug("https://TOKEN@github.com/hewigovens/jayjay.git").as_deref(),
            expected
        );
    }

    #[test]
    fn encode_path_segment_escapes_specials_and_preserves_slash() {
        assert_eq!(encode_path_segment("feat/foo"), "feat/foo");
        assert_eq!(encode_path_segment("weird#name?yes"), "weird%23name%3Fyes");
    }

    #[test]
    fn slug_rejects_non_github_and_malformed() {
        // Lookalike hosts must not pass — opening a bogus PR URL is the failure mode here.
        assert_eq!(
            github_slug("https://github.com.evil.org/hewigovens/jayjay"),
            None
        );
        assert_eq!(github_slug("https://evilgithub.com/foo/bar"), None);
        assert_eq!(
            github_slug("https://gitlab.com/hewigovens/jayjay.git"),
            None
        );
        assert_eq!(github_slug("https://github.com/lonely"), None);
        assert_eq!(github_slug(""), None);
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
        assert_eq!(
            parse_gh_pr_json(json).unwrap().checks,
            ChecksStatus::Pending
        );
    }
}
