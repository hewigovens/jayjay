use jayjay_network::Fetch;

use super::super::PrLookup;
use super::api::Api;
use super::pull_request::OriginPull;
use super::status::OriginCheckRun;
use crate::repo::forge_auth::TokenProvider;
use crate::repo::hosted_repo::HostedRepo;
use crate::types::{ChecksStatus, PrState};

pub(crate) fn pr_info(remote: &HostedRepo, bookmark: &str) -> PrLookup {
    lookup(&Api::for_remote(remote), bookmark)
}

fn lookup(api: &Api<impl Fetch, impl TokenProvider>, bookmark: &str) -> PrLookup {
    match find_pull(api, bookmark) {
        Ok(Some((pull, state))) => {
            let checks = pull
                .head_sha()
                .and_then(|sha| api.check_runs(sha).ok())
                .map_or(ChecksStatus::None, |runs| OriginCheckRun::rollup(&runs));
            PrLookup::Found(pull.into_pr_info(state, checks))
        }
        Ok(None) => PrLookup::NotFound,
        Err(_) => PrLookup::Unknown,
    }
}

pub(crate) fn open_pr(
    api: &Api<impl Fetch, impl TokenProvider>,
    bookmark: &str,
) -> Result<Option<(u32, String)>, String> {
    Ok(find_pull(api, bookmark)?
        .filter(|(_, state)| *state == PrState::Open)
        .map(|(pull, _)| (pull.number, pull.web_url)))
}

/// The open pull request, else the newest one. `Err` means the lookup failed, so nothing may be created.
fn find_pull(
    api: &Api<impl Fetch, impl TokenProvider>,
    bookmark: &str,
) -> Result<Option<(OriginPull, PrState)>, String> {
    let mut fallback = None;
    for pull in api.pulls(bookmark).map_err(|error| error.to_string())? {
        let state = pull.pr_state()?;
        if state == PrState::Open {
            return Ok(Some((pull, state)));
        }
        fallback.get_or_insert((pull, state));
    }
    Ok(fallback)
}

#[cfg(test)]
mod tests {
    use jayjay_network::{MockFetch, NetError, Request, Response};

    use super::*;
    use crate::repo::forge_auth::{ForgeToken, TokenSource};
    use crate::repo::hosted_repo::RepoHost;

    fn run(pulls: &'static str, checks: &'static str) -> PrLookup {
        let fetch = MockFetch::new(move |request: &Request| {
            if request.url.contains("/check-runs") {
                Response::ok(200, checks)
            } else {
                Response::ok(200, pulls)
            }
        });
        lookup(&api(&fetch), "feat/x")
    }

    fn api<F: Fetch>(fetch: F) -> Api<F, Option<ForgeToken>> {
        let token = Some(ForgeToken {
            secret: "t".into(),
            source: TokenSource::Cli("origin"),
        });
        let remote = HostedRepo {
            host: RepoHost::Cursor,
            owner: "o".into(),
            repo: "r".into(),
        };
        Api::new(fetch, token, &remote)
    }

    #[test]
    fn prefers_the_open_pull_and_rolls_up_its_checks() {
        let pulls = r#"{"pullRequests":[
            {"number":"9","state":"closed","merged":true,"title":"old","webUrl":"https://cursor.com/codebase/o/r/pull/9"},
            {"number":"12","state":"open","draft":true,"title":"wip","webUrl":"https://cursor.com/codebase/o/r/pull/12","head":{"ref":"feat/x","sha":"abc"}}
        ]}"#;
        let checks =
            r#"{"checkRuns":[{"status":"completed","conclusion":"success"},{"status":"failing"}]}"#;
        let PrLookup::Found(pr) = run(pulls, checks) else {
            panic!("expected a pull request");
        };
        assert_eq!((pr.number, pr.state), (12, PrState::Open));
        assert_eq!(pr.checks, ChecksStatus::Pending);

        let merged = r#"{"pullRequests":[{"number":"9","state":"closed","merged":true,"title":"old","webUrl":"u"}]}"#;
        let PrLookup::Found(pr) = run(merged, "{}") else {
            panic!("expected a pull request");
        };
        assert_eq!((pr.state, pr.checks), (PrState::Merged, ChecksStatus::None));
    }

    #[test]
    fn check_rollup_includes_later_pages_and_discards_partial_results() {
        let pulls = r#"{"pullRequests":[{"number":"1","state":"open","title":"t","webUrl":"u","head":{"sha":"abc"}}]}"#;
        let first_page = serde_json::json!({
            "checkRuns": vec![serde_json::json!({"status":"completed","conclusion":"success"}); 100],
            "nextPageToken": "next/+?=&",
        })
        .to_string();
        for (last_page, expected) in [
            (
                Some(r#"{"checkRuns":[{"status":"completed","conclusion":"failure"}]}"#),
                ChecksStatus::Failing,
            ),
            (
                Some(r#"{"checkRuns":[{"status":"in_progress"}]}"#),
                ChecksStatus::Pending,
            ),
            (None, ChecksStatus::None),
        ] {
            let fetch = MockFetch::new(|request: &Request| {
                if !request.url.contains("/check-runs") {
                    Response::ok(200, pulls)
                } else if request.url.contains("pageToken=") {
                    assert!(request.url.ends_with("&pageToken=next%2F%2B%3F%3D%26"));
                    match last_page {
                        Some(body) => Response::ok(200, body),
                        None => Err(NetError::Transport),
                    }
                } else {
                    Response::ok(200, first_page.clone())
                }
            });
            let PrLookup::Found(pr) = lookup(&api(&fetch), "feat/x") else {
                panic!("expected a pull request");
            };
            assert_eq!(pr.checks, expected);
        }
    }

    #[test]
    fn only_a_confirmed_empty_list_is_absence() {
        assert!(matches!(
            run(r#"{"pullRequests":[]}"#, "{}"),
            PrLookup::NotFound
        ));
        assert!(matches!(
            run(
                r#"{"pullRequests":[{"number":"1","state":"superseded","title":"t","webUrl":"u"}]}"#,
                "{}"
            ),
            PrLookup::Unknown
        ));
        let offline = MockFetch::new(|_| Err(NetError::Transport));
        assert!(matches!(
            lookup(&api(&offline), "feat/x"),
            PrLookup::Unknown
        ));
    }
}
