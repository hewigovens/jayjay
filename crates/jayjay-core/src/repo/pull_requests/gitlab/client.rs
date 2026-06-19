use jayjay_network::Auth;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

use super::super::PrLookup;
use super::merge_request::GitLabMrResponse;
use super::status::GitLabCommitStatus;
use crate::repo::hosted_repo::HostedRepo;
use crate::types::ChecksStatus;

const GITLAB_API_URL: &str = "https://gitlab.com/api/v4";
const URL_COMPONENT_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

pub(crate) fn pr_info(remote: &HostedRepo, bookmark: &str) -> PrLookup {
    let auth = gitlab_auth();
    // GitLab filters by source branch server-side, so a single request suffices.
    let url = merge_requests_url(remote, bookmark);
    let body = match jayjay_network::get_text_with_auth(&url, &auth) {
        Ok(body) => body,
        // A private project 404s and rate limits 429; neither is a confirmed "no MR".
        Err(_) => return PrLookup::Unknown,
    };
    let Ok(mrs) = serde_json::from_str::<Vec<GitLabMrResponse>>(&body) else {
        return PrLookup::Unknown;
    };
    let Some(mr) = pick_mr(mrs, bookmark) else {
        return PrLookup::NotFound;
    };
    let checks = mr
        .head_sha()
        .and_then(|sha| commit_status(remote, sha, &auth))
        .unwrap_or(ChecksStatus::None);
    PrLookup::Found(mr.into_pr_info(checks))
}

/// Token from `GITLAB_TOKEN`; without it private projects 404 and rate limits apply.
fn gitlab_auth() -> Auth {
    Auth::bearer(std::env::var("GITLAB_TOKEN").ok())
}

/// Prefer an open MR for the branch; fall back to the most recent other state.
/// The listing is ordered newest-first, so the first match is the freshest.
fn pick_mr(mrs: Vec<GitLabMrResponse>, source_branch: &str) -> Option<GitLabMrResponse> {
    let mut fallback = None;
    for mr in mrs.into_iter().filter(|mr| mr.matches(source_branch)) {
        if mr.is_open() {
            return Some(mr);
        }
        fallback.get_or_insert(mr);
    }
    fallback
}

fn merge_requests_url(remote: &HostedRepo, source_branch: &str) -> String {
    format!(
        "{}/projects/{}/merge_requests?source_branch={}&state=all&order_by=created_at&sort=desc",
        GITLAB_API_URL,
        project_id(remote),
        encode(source_branch),
    )
}

fn commit_status(remote: &HostedRepo, sha: &str, auth: &Auth) -> Option<ChecksStatus> {
    let url = format!(
        "{}/projects/{}/repository/commits/{}",
        GITLAB_API_URL,
        project_id(remote),
        encode(sha),
    );
    let body = jayjay_network::get_text_with_auth(&url, auth).ok()?;
    let commit: GitLabCommitStatus = serde_json::from_str(&body).ok()?;
    Some(commit.checks())
}

/// GitLab identifies a project by URL-encoded `namespace/project` (slash escaped).
fn project_id(remote: &HostedRepo) -> String {
    encode(&format!("{}/{}", remote.owner, remote.repo))
}

fn encode(s: &str) -> String {
    utf8_percent_encode(s, URL_COMPONENT_ENCODE_SET).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::hosted_repo::RepoHost;

    fn remote() -> HostedRepo {
        HostedRepo {
            host: RepoHost::GitLab,
            owner: "owner".into(),
            repo: "repo".into(),
        }
    }

    fn mrs(json: &str) -> Vec<GitLabMrResponse> {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn merge_requests_url_escapes_project_path_and_branch() {
        let url = merge_requests_url(&remote(), "feat/foo");
        assert!(url.starts_with("https://gitlab.com/api/v4/projects/owner%2Frepo/merge_requests?"));
        assert!(url.contains("source_branch=feat%2Ffoo"));
        assert!(url.contains("state=all"));
    }

    #[test]
    fn pick_prefers_open_over_merged() {
        let chosen = pick_mr(
            mrs(r#"[
                {"iid":1,"state":"merged","title":"t","web_url":"u","source_branch":"b","sha":""},
                {"iid":2,"state":"opened","title":"t","web_url":"u","source_branch":"b","sha":""}
            ]"#),
            "b",
        )
        .unwrap();
        assert_eq!(chosen.into_pr_info(ChecksStatus::None).number, 2);
    }

    #[test]
    fn pick_falls_back_to_first_when_none_open() {
        let chosen = pick_mr(
            mrs(r#"[
                {"iid":5,"state":"closed","title":"t","web_url":"u","source_branch":"b","sha":""},
                {"iid":3,"state":"merged","title":"t","web_url":"u","source_branch":"b","sha":""}
            ]"#),
            "b",
        )
        .unwrap();
        assert_eq!(chosen.into_pr_info(ChecksStatus::None).number, 5);
    }

    #[test]
    fn pick_ignores_other_branches() {
        let chosen = pick_mr(
            mrs(
                r#"[{"iid":1,"state":"opened","title":"t","web_url":"u","source_branch":"other","sha":""}]"#,
            ),
            "b",
        );
        assert!(chosen.is_none());
    }
}
