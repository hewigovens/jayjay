use super::super::github::GhPrResponse;
use super::plan::{PrHeadRepo, ResolvedPullRequest};
use super::url::ParsedPullRequestUrl;
use crate::repo::Repo;
use crate::repo::environment::gh_binary;
use crate::repo::hosted_repo::{HostedRepo, RepoHost};
use crate::types::{CoreError, CoreResult};

const UNUSABLE_HEAD: &str =
    "GitHub did not return a head to fetch for this pull request; the fork may have been deleted.";

pub(super) fn resolve(
    repo: &Repo,
    parsed: &ParsedPullRequestUrl,
) -> CoreResult<ResolvedPullRequest> {
    let canonical_url = format!("{}/pull/{}", parsed.base.web_url(), parsed.number);
    let args = [
        "pr",
        "view",
        "--json",
        "number,title,url,state,headRefName,headRefOid,headRepositoryOwner,headRepository,isCrossRepository",
        "--",
        &canonical_url,
    ];
    let output = repo.cancellable_output(&gh_binary(), &args, "gh pr view")?;
    if !output.status.success() {
        let detail = Repo::stderr_text(&output);
        return Err(CoreError::internal(format!(
            "Couldn't load the pull request with gh: {detail}"
        )));
    }
    serde_json::from_str::<GhPrResponse>(&Repo::stdout_text(&output))
        .ok()
        .and_then(ResolvedPullRequest::from_gh)
        .ok_or_else(|| CoreError::internal(UNUSABLE_HEAD))
}

impl ResolvedPullRequest {
    /// None when a fork PR lost its head repository, which is how GitHub reports a deleted fork.
    fn from_gh(pr: GhPrResponse) -> Option<Self> {
        let head = if pr.is_cross_repository {
            PrHeadRepo::Fork(HostedRepo {
                host: RepoHost::GitHub,
                owner: pr.head_repository_owner?.login,
                repo: pr.head_repository?.name,
            })
        } else {
            PrHeadRepo::SameRepository
        };
        Some(Self {
            number: pr.number,
            title: pr.title,
            state: pr.state.into(),
            url: pr.url,
            head_branch: pr.head_ref_name,
            head_commit_id: pr.head_ref_oid,
            head,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fork_pr_resolves_to_the_fork_urls_and_a_deleted_fork_is_an_error() {
        let json = r#"{
            "number": 290, "title": "Fix it", "url": "https://github.com/o/r/pull/290",
            "state": "OPEN", "headRefName": "feat/x", "headRefOid": "abc123",
            "isCrossRepository": true,
            "headRepositoryOwner": {"login": "alice"}, "headRepository": {"name": "r"}
        }"#;
        let from_json = |json: &str| {
            ResolvedPullRequest::from_gh(serde_json::from_str::<GhPrResponse>(json).unwrap())
        };
        let resolved = from_json(json).expect("a fork head");
        assert_eq!(resolved.head_branch, "feat/x");
        let PrHeadRepo::Fork(fork) = resolved.head else {
            panic!("expected a fork head");
        };
        assert_eq!(fork.clone_url(false), "https://github.com/alice/r.git");
        assert_eq!(fork.clone_url(true), "git@github.com:alice/r.git");

        let deleted = json
            .replace(r#"{"login": "alice"}"#, "null")
            .replace(r#"{"name": "r"}"#, "null");
        assert!(from_json(&deleted).is_none());
    }
}
