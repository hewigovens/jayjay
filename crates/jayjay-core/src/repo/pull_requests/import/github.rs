use jayjay_network::Fetch;
use serde::Deserialize;
use serde_json::json;

use super::super::github::{Api, GitHubPullRequest, PULL_REQUEST_FIELDS};
use super::plan::{ForkRepo, PrHeadRepo, ResolvedPullRequest};
use super::url::ParsedPullRequestUrl;
use crate::repo::forge_auth::TokenProvider;
use crate::repo::hosted_repo::{HostedRepo, RepoHost};
use crate::types::{JayError, JayResult};

const UNUSABLE_HEAD: &str =
    "GitHub did not return a head to fetch for this pull request; the fork may have been deleted.";

pub(super) fn resolve(parsed: &ParsedPullRequestUrl, ssh: bool) -> JayResult<ResolvedPullRequest> {
    resolve_with(&Api::for_remote(&parsed.base), parsed.number, ssh)
}

fn resolve_with(
    api: &Api<impl Fetch, impl TokenProvider>,
    number: u32,
    ssh: bool,
) -> JayResult<ResolvedPullRequest> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Data {
        repository: Option<Repository>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Repository {
        pull_request: Option<GitHubPullRequest>,
    }

    let hint = RepoHost::GitHub.sign_in_hint();
    if !api.client().has_token() {
        return Err(JayError::internal(format!(
            "Loading a pull request needs a GitHub login. {hint}"
        )));
    }
    let query = format!(
        "query($owner: String!, $repo: String!, $number: Int!) {{
  repository(owner: $owner, name: $repo) {{ pullRequest(number: $number) {{ ...PullRequestFields }} }}
}}{PULL_REQUEST_FIELDS}"
    );
    let data: Data = api.graphql(&query, json!({ "number": number })).map_err(
        |error| match error.status() {
            Some(401 | 404) => JayError::internal(format!(
                "GitHub couldn't find this pull request with the current login ({error}). {hint}"
            )),
            _ => JayError::internal(format!(
                "Couldn't load the pull request from GitHub: {error}"
            )),
        },
    )?;
    data.repository
        .and_then(|repository| repository.pull_request)
        .and_then(|pr| ResolvedPullRequest::from_github(pr, ssh))
        .ok_or_else(|| JayError::internal(UNUSABLE_HEAD))
}

impl ResolvedPullRequest {
    /// None when a fork PR lost its head repository, which is how GitHub reports a deleted fork.
    fn from_github(pr: GitHubPullRequest, ssh: bool) -> Option<Self> {
        let head = if pr.is_cross_repository {
            let fork = HostedRepo {
                host: RepoHost::GitHub,
                owner: pr.head_repository_owner?.login,
                repo: pr.head_repository?.name,
            };
            PrHeadRepo::Fork(ForkRepo {
                clone_url: fork.clone_url(ssh),
                name_hint: fork.owner,
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
    use jayjay_network::{MockFetch, Response};

    use super::super::super::github::pull_request_json;
    use super::*;
    use crate::repo::forge_auth::{ForgeToken, TokenSource};

    fn base() -> HostedRepo {
        HostedRepo {
            host: RepoHost::GitHub,
            owner: "o".into(),
            repo: "r".into(),
        }
    }

    fn resolve_json(pr_json: &str, ssh: bool) -> JayResult<ResolvedPullRequest> {
        let body = format!(r#"{{"data":{{"repository":{{"pullRequest":{pr_json}}}}}}}"#);
        let fetch = MockFetch::new(|_| Response::ok(200, body.clone()));
        let token = Some(ForgeToken {
            secret: "t".into(),
            source: TokenSource::Environment("GH_TOKEN"),
        });
        resolve_with(&Api::new(&fetch, token, &base()), 290, ssh)
    }

    #[test]
    fn fork_pr_resolves_to_the_fork_urls_and_a_deleted_fork_is_an_error() {
        let json = pull_request_json(290, "OPEN", "[]")
            .replace(
                r#""isCrossRepository":false"#,
                r#""isCrossRepository":true"#,
            )
            .replace(r#"{"login":"o"}"#, r#"{"login":"alice"}"#);
        let resolved = resolve_json(&json, false).expect("a fork head");
        assert_eq!(resolved.head_branch, "feat");
        assert_eq!(
            resolved.head,
            PrHeadRepo::Fork(ForkRepo {
                name_hint: "alice".into(),
                clone_url: "https://github.com/alice/r.git".into(),
            })
        );
        let PrHeadRepo::Fork(fork) = resolve_json(&json, true).expect("ssh head").head else {
            panic!("expected a fork head");
        };
        assert_eq!(fork.clone_url, "git@github.com:alice/r.git");

        let deleted = json.replace(r#"{"login":"alice"}"#, "null");
        assert!(resolve_json(&deleted, false).is_err());
        assert!(resolve_json("null", false).is_err());
    }

    #[test]
    fn a_missing_login_names_how_to_sign_in() {
        let fetch = MockFetch::new(|_| Response::ok(200, "{}"));
        let error = resolve_with(&Api::new(&fetch, None, &base()), 290, false).unwrap_err();
        assert!(error.to_string().contains("gh auth login"));
        assert!(fetch.requests().is_empty());
    }
}
