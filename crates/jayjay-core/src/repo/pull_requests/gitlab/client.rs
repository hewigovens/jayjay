use jayjay_network::{Fetch, HttpClient, encode_path_segment};

use super::super::{ForgeClient, PrLookup};
use super::merge_request::GitLabMrResponse;
use super::status::GitLabCommitStatus;
use crate::repo::forge_auth::TokenProvider;
use crate::repo::hosted_repo::{HostedRepo, RepoHost};
use crate::types::ChecksStatus;

pub(crate) fn pr_info(remote: &HostedRepo, bookmark: &str) -> PrLookup {
    lookup(
        &ForgeClient::gitlab(HttpClient::default(), RepoHost::GitLab),
        remote,
        bookmark,
    )
}

fn lookup(
    api: &ForgeClient<impl Fetch, impl TokenProvider>,
    remote: &HostedRepo,
    bookmark: &str,
) -> PrLookup {
    // GitLab filters by source branch server-side, so a single request suffices.
    let Ok(mrs) = api.get::<Vec<GitLabMrResponse>>(&branch_merge_requests_path(remote, bookmark))
    else {
        // A private project 404s and rate limits 429; neither is a confirmed "no MR".
        return PrLookup::Unknown;
    };
    let Some(mr) = pick_mr(mrs, bookmark) else {
        return PrLookup::NotFound;
    };
    let checks = mr
        .head_sha()
        .and_then(|sha| commit_status(api, remote, sha))
        .unwrap_or(ChecksStatus::None);
    PrLookup::Found(mr.into_pr_info(checks))
}

// The listing is newest-first, so the first match is the freshest when no MR is open.
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

pub(crate) fn merge_requests_path(remote: &HostedRepo) -> String {
    format!("projects/{}/merge_requests", project_id(remote))
}

fn branch_merge_requests_path(remote: &HostedRepo, source_branch: &str) -> String {
    format!(
        "{}?source_branch={}&state=all&order_by=created_at&sort=desc",
        merge_requests_path(remote),
        encode_path_segment(source_branch),
    )
}

pub(crate) fn merge_request_path(remote: &HostedRepo, iid: u32) -> String {
    format!("{}/{iid}", merge_requests_path(remote))
}

pub(crate) fn project_path(id: u64) -> String {
    format!("projects/{id}")
}

fn commit_status(
    api: &ForgeClient<impl Fetch, impl TokenProvider>,
    remote: &HostedRepo,
    sha: &str,
) -> Option<ChecksStatus> {
    let path = format!(
        "projects/{}/repository/commits/{}",
        project_id(remote),
        encode_path_segment(sha),
    );
    api.get::<GitLabCommitStatus>(&path)
        .ok()
        .map(|commit| commit.checks())
}

fn project_id(remote: &HostedRepo) -> String {
    encode_path_segment(&format!("{}/{}", remote.owner, remote.repo))
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
    fn merge_requests_path_escapes_project_path_and_branch() {
        let path = branch_merge_requests_path(&remote(), "feat/foo");
        assert!(path.starts_with("projects/owner%2Frepo/merge_requests?"));
        assert!(path.contains("source_branch=feat%2Ffoo"));
        assert!(path.contains("state=all"));
    }

    #[test]
    fn detail_paths_address_the_target_project_and_numeric_source_project() {
        let nested = HostedRepo {
            host: RepoHost::GitLab,
            owner: "group/sub".into(),
            repo: "base".into(),
        };
        assert_eq!(
            merge_request_path(&nested, 45),
            "projects/group%2Fsub%2Fbase/merge_requests/45"
        );
        assert_eq!(project_path(83542242), "projects/83542242");
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
