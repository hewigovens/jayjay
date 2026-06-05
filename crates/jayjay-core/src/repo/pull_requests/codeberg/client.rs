use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

use super::pull_request::CodebergPrResponse;
use super::status::CodebergCombinedStatus;
use crate::repo::hosted_repo::HostedRepo;
use crate::types::{ChecksStatus, PrInfo};

const CODEBERG_API_URL: &str = "https://codeberg.org/api/v1";
const URL_COMPONENT_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

pub(crate) fn pr_info(remote: &HostedRepo, bookmark: &str) -> Option<PrInfo> {
    // Match by head only; an existing PR counts regardless of its base branch.
    let url = format!(
        "{}/repos/{}/{}/pulls?state=all&head={}",
        CODEBERG_API_URL,
        encode(&remote.owner),
        encode(&remote.repo),
        encode(bookmark)
    );
    let body = jayjay_network::get_text(&url)?;
    let pr = serde_json::from_str::<Vec<CodebergPrResponse>>(&body)
        .ok()?
        .into_iter()
        .find(|pr| pr.matches(bookmark))?;
    let checks = pr
        .head_sha()
        .and_then(|sha| commit_status(remote, sha))
        .unwrap_or(ChecksStatus::None);
    Some(pr.into_pr_info(checks))
}

fn commit_status(remote: &HostedRepo, sha: &str) -> Option<ChecksStatus> {
    let url = format!(
        "{}/repos/{}/{}/commits/{}/status",
        CODEBERG_API_URL,
        encode(&remote.owner),
        encode(&remote.repo),
        encode(sha)
    );
    let body = jayjay_network::get_text(&url)?;
    let combined: CodebergCombinedStatus = serde_json::from_str(&body).ok()?;
    Some(combined.checks())
}

fn encode(s: &str) -> String {
    utf8_percent_encode(s, URL_COMPONENT_ENCODE_SET).to_string()
}
