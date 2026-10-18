use jayjay_network::{Fetch, Method, encode_path_segment};
use serde::Deserialize;
use serde_json::json;

use super::super::forge::{ForgeTarget, created_with_number, failed, non_empty_or, updated};
use crate::repo::forge_auth::TokenProvider;
use crate::repo::pull_requests::ApiError;
use crate::repo::pull_requests::github::Api;
use crate::types::SubmittedLayer;

#[derive(Deserialize)]
struct RestPull {
    number: u32,
    html_url: String,
}

pub(crate) fn create_or_update_pr(
    api: &Api<impl Fetch, impl TokenProvider>,
    target: &ForgeTarget,
) -> SubmittedLayer {
    match open_pr(api, &target.bookmark) {
        Ok(Some(pull)) => edit_pr(api, target, pull),
        Ok(None) => create_pr(api, target),
        Err(error) => failed(target, error.to_string()),
    }
}

fn open_pr(
    api: &Api<impl Fetch, impl TokenProvider>,
    bookmark: &str,
) -> Result<Option<RestPull>, ApiError> {
    let head = encode_path_segment(&format!("{}:{bookmark}", api.owner()));
    let pulls: Vec<RestPull> = api
        .client()
        .get(&api.repo_path(&format!("pulls?state=open&head={head}&per_page=1")))?;
    Ok(pulls.into_iter().next())
}

fn edit_pr(
    api: &Api<impl Fetch, impl TokenProvider>,
    target: &ForgeTarget,
    pull: RestPull,
) -> SubmittedLayer {
    let title = non_empty_or(&target.title, &target.bookmark);
    let body = json!({ "base": target.base, "title": title, "body": target.body });
    let path = api.repo_path(&format!("pulls/{}", pull.number));
    match api.client().send::<RestPull>(Method::Patch, &path, &body) {
        Ok(_) => updated(
            target,
            title,
            pull.number,
            pull.html_url,
            format!("PR #{}", pull.number),
        ),
        Err(error) => failed(target, error.to_string()),
    }
}

fn create_pr(api: &Api<impl Fetch, impl TokenProvider>, target: &ForgeTarget) -> SubmittedLayer {
    let title = non_empty_or(&target.title, &target.bookmark);
    let body = json!({
        "head": target.bookmark,
        "base": target.base,
        "title": title,
        "body": target.body,
    });
    match api
        .client()
        .send::<RestPull>(Method::Post, &api.repo_path("pulls"), &body)
    {
        Ok(pull) => created_with_number(target, title, pull.number, pull.html_url),
        Err(error) => failed(target, error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use jayjay_network::{MockFetch, NetError, Request, Response};

    use super::*;
    use crate::repo::forge_auth::{ForgeToken, TokenSource};
    use crate::repo::hosted_repo::{HostedRepo, RepoHost};
    use crate::types::StackLayerOutcome;

    fn api<F: Fetch>(fetch: F) -> Api<F, Option<ForgeToken>> {
        let remote = HostedRepo {
            host: RepoHost::GitHub,
            owner: "o".into(),
            repo: "r".into(),
        };
        let token = Some(ForgeToken {
            secret: "t".into(),
            source: TokenSource::Environment("GH_TOKEN"),
        });
        Api::new(fetch, token, &remote)
    }

    fn target() -> ForgeTarget {
        ForgeTarget {
            bookmark: "feat/x".into(),
            base: "main".into(),
            title: "Add x".into(),
            body: "Body".into(),
        }
    }

    const PULL: &str = r#"{"number":12,"html_url":"https://github.com/o/r/pull/12"}"#;

    #[test]
    fn creates_a_pull_request_when_no_open_one_exists() {
        let fetch = MockFetch::new(|request: &Request| match request.method {
            Method::Get => Response::ok(200, "[]"),
            _ => Response::ok(201, PULL),
        });
        let layer = create_or_update_pr(&api(&fetch), &target());
        assert_eq!(layer.outcome, StackLayerOutcome::Created);
        assert_eq!(
            (layer.pr_number, layer.pr_url.as_str()),
            (12, "https://github.com/o/r/pull/12")
        );
        assert!(
            fetch.requests()[0]
                .url
                .ends_with("pulls?state=open&head=o%3Afeat%2Fx&per_page=1")
        );
        let writes = fetch.writes();
        assert_eq!(writes[0].url, "https://api.github.com/repos/o/r/pulls");
        let body: serde_json::Value = serde_json::from_str(writes[0].body_text()).unwrap();
        assert_eq!(
            body,
            json!({ "head": "feat/x", "base": "main", "title": "Add x", "body": "Body" })
        );
    }

    #[test]
    fn retargets_an_open_pull_request() {
        let fetch = MockFetch::new(|request: &Request| match request.method {
            Method::Get => Response::ok(200, format!("[{PULL}]")),
            _ => Response::ok(200, PULL),
        });
        let layer = create_or_update_pr(&api(&fetch), &target());
        assert_eq!(layer.outcome, StackLayerOutcome::Updated);
        assert_eq!(layer.detail, "PR #12 → base main");
        let writes = fetch.writes();
        assert_eq!(
            (writes[0].method, writes[0].url.as_str()),
            (Method::Patch, "https://api.github.com/repos/o/r/pulls/12")
        );
    }

    #[test]
    fn failures_carry_the_api_message() {
        let fetch = MockFetch::new(|request: &Request| match request.method {
            Method::Get => Response::ok(200, "[]"),
            _ => Response::ok(422, r#"{"message":"Validation Failed"}"#),
        });
        let layer = create_or_update_pr(&api(&fetch), &target());
        assert_eq!(layer.outcome, StackLayerOutcome::Failed);
        assert!(
            layer.detail.contains("Validation Failed"),
            "{}",
            layer.detail
        );

        let offline = MockFetch::new(|_| Err(NetError::Transport));
        assert!(
            api(&offline)
                .client()
                .preflight()
                .unwrap_err()
                .to_string()
                .contains("gh auth login")
        );
    }
}
