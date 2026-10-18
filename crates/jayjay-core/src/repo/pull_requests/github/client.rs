use jayjay_network::Fetch;
use serde::Deserialize;
use serde_json::json;

use super::super::PrLookup;
use super::api::Api;
use super::pull_request::{GitHubPullRequest, Nodes, PULL_REQUEST_FIELDS};
use crate::repo::forge_auth::TokenProvider;
use crate::repo::hosted_repo::HostedRepo;

/// Looks only at origin, so a fork clone misses its PRs to upstream.
pub(crate) fn pr_info(remote: &HostedRepo, bookmark: &str) -> PrLookup {
    lookup(&Api::for_remote(remote), bookmark)
}

fn lookup(api: &Api<impl Fetch, impl TokenProvider>, bookmark: &str) -> PrLookup {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Data {
        repository: Option<Repository>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Repository {
        pull_requests: Nodes<GitHubPullRequest>,
    }

    // GraphQL has no anonymous access, so a missing login can't confirm an absence.
    if !api.client().has_token() {
        return PrLookup::Unknown;
    }
    let query = format!(
        "query($owner: String!, $repo: String!, $head: String!) {{
  repository(owner: $owner, name: $repo) {{
    pullRequests(headRefName: $head, first: 100, orderBy: {{field: CREATED_AT, direction: DESC}}) {{
      nodes {{ ...PullRequestFields }}
    }}
  }}
}}{PULL_REQUEST_FIELDS}"
    );
    let Ok(data) = api.graphql::<Data>(&query, json!({ "head": bookmark })) else {
        return PrLookup::Unknown;
    };
    let Some(repository) = data.repository else {
        return PrLookup::Unknown;
    };
    match pick_pull_request(repository.pull_requests.nodes) {
        Some(pr) => PrLookup::Found(pr.into_pr_info()),
        None => PrLookup::NotFound,
    }
}

/// A fork's branch can share the bookmark's name.
fn pick_pull_request(prs: Vec<GitHubPullRequest>) -> Option<GitHubPullRequest> {
    let mut fallback = None;
    for pr in prs.into_iter().filter(|pr| !pr.is_cross_repository) {
        if pr.is_open() {
            return Some(pr);
        }
        fallback.get_or_insert(pr);
    }
    fallback
}

#[cfg(test)]
mod tests {
    use jayjay_network::{MockFetch, NetError, Response};

    use super::super::pull_request::pull_request_json;
    use super::*;
    use crate::repo::forge_auth::{ForgeToken, TokenSource};
    use crate::repo::hosted_repo::RepoHost;
    use crate::types::PrState;

    fn remote() -> HostedRepo {
        HostedRepo {
            host: RepoHost::GitHub,
            owner: "o".into(),
            repo: "r".into(),
        }
    }

    fn token() -> Option<ForgeToken> {
        Some(ForgeToken {
            secret: "t".into(),
            source: TokenSource::Environment("GH_TOKEN"),
        })
    }

    fn lookup_with(
        token: Option<ForgeToken>,
        respond: impl Fn() -> Result<Response, NetError>,
    ) -> PrLookup {
        let fetch = MockFetch::new(|request| {
            assert!(request.body_text().contains(r#""head":"feat""#));
            respond()
        });
        lookup(&Api::new(&fetch, token, &remote()), "feat")
    }

    fn page(nodes: &[String]) -> String {
        format!(
            r#"{{"data":{{"repository":{{"pullRequests":{{"nodes":[{}]}}}}}}}}"#,
            nodes.join(",")
        )
    }

    #[test]
    fn an_open_same_repository_pull_request_beats_newer_closed_and_fork_ones() {
        let fork = pull_request_json(9, "OPEN", "[]").replace(
            r#""isCrossRepository":false"#,
            r#""isCrossRepository":true"#,
        );
        let body = page(&[
            fork,
            pull_request_json(8, "CLOSED", "[]"),
            pull_request_json(7, "OPEN", "[]"),
        ]);
        let PrLookup::Found(pr) = lookup_with(token(), || Response::ok(200, body.clone())) else {
            panic!("expected a pull request");
        };
        assert_eq!((pr.number, pr.state), (7, PrState::Open));
    }

    #[test]
    fn only_an_answered_empty_listing_confirms_absence() {
        assert!(matches!(
            lookup_with(token(), || Response::ok(200, page(&[]))),
            PrLookup::NotFound
        ));
        let failures: [fn() -> Result<Response, NetError>; 4] = [
            || Response::ok(401, r#"{"message":"Bad credentials"}"#),
            || Response::ok(502, ""),
            || Err(NetError::Transport),
            || {
                Response::ok(
                    200,
                    r#"{"errors":[{"type":"NOT_FOUND","message":"no repo"}]}"#,
                )
            },
        ];
        for respond in failures {
            assert!(matches!(lookup_with(token(), respond), PrLookup::Unknown));
        }
        assert!(matches!(
            lookup_with(None, || Response::ok(200, page(&[]))),
            PrLookup::Unknown
        ));
    }
}
