use jayjay_network::Fetch;

use super::forge::{ForgeTarget, created_with_number, failed, non_empty_or, updated};
use crate::repo::forge_auth::TokenProvider;
use crate::repo::pull_requests::cursor::{self, Api, UpdatePull};
use crate::types::{JayError, JayResult, SubmittedLayer};

pub(super) fn preflight(api: &Api) -> JayResult<String> {
    api.client().preflight()?;
    api.pull_request_base().map_err(JayError::internal)
}

pub(super) fn create_or_update_pr(
    api: &Api<impl Fetch, impl TokenProvider>,
    target: &ForgeTarget,
) -> SubmittedLayer {
    let title = non_empty_or(&target.title, &target.bookmark);
    match cursor::open_pr(api, &target.bookmark) {
        Ok(Some((number, url))) => {
            let edit = UpdatePull {
                title: &title,
                body: &target.body,
                base: &target.base,
            };
            match api.update_pull(number, &edit) {
                Ok(()) => updated(target, title, number, url, format!("PR #{number}")),
                Err(error) => failed(target, error.to_string()),
            }
        }
        Ok(None) => match api.create_pull(&target.bookmark, &target.base, &title, &target.body) {
            Ok(pull) => created_with_number(target, title, pull.number, pull.web_url),
            Err(detail) => failed(target, detail),
        },
        Err(detail) => failed(target, detail),
    }
}

#[cfg(test)]
mod tests {
    use jayjay_network::{Method, MockFetch, Request, Response};

    use super::*;
    use crate::repo::forge_auth::{ForgeToken, TokenSource};
    use crate::repo::hosted_repo::{HostedRepo, RepoHost};
    use crate::types::StackLayerOutcome;

    fn submit(listed: &'static str) -> (SubmittedLayer, Vec<Request>) {
        let fetch = MockFetch::new(move |request: &Request| match request.method {
            Method::Get => Response::ok(200, listed),
            _ => Response::ok(
                200,
                r#"{"number":"7","state":"open","title":"Add x","webUrl":"https://cursor.com/codebase/o/r/pull/7"}"#,
            ),
        });
        let token = Some(ForgeToken {
            secret: "t".into(),
            source: TokenSource::Cli("origin"),
        });
        let remote = HostedRepo {
            host: RepoHost::Cursor,
            owner: "o".into(),
            repo: "r".into(),
        };
        let target = ForgeTarget {
            bookmark: "feat/x".into(),
            base: "feat/base".into(),
            title: "Add x".into(),
            body: "body".into(),
        };
        let layer = create_or_update_pr(&Api::new(&fetch, token, &remote), &target);
        (layer, fetch.writes())
    }

    fn body(request: &Request) -> serde_json::Value {
        serde_json::from_str(request.body_text()).unwrap()
    }

    #[test]
    fn creates_a_missing_layer_and_retargets_an_open_one() {
        let (layer, writes) = submit(r#"{"pullRequests":[]}"#);
        assert_eq!(
            (layer.outcome, layer.pr_number),
            (StackLayerOutcome::Created, 7)
        );
        assert_eq!(writes[0].method, Method::Post);
        assert_eq!(body(&writes[0])["head"], "feat/x");
        assert_eq!(body(&writes[0])["base"], "feat/base");

        let (layer, writes) = submit(
            r#"{"pullRequests":[{"number":"7","state":"open","title":"old","webUrl":"https://cursor.com/codebase/o/r/pull/7"}]}"#,
        );
        assert_eq!(
            (layer.outcome, layer.pr_number),
            (StackLayerOutcome::Updated, 7)
        );
        assert_eq!(writes[0].method, Method::Patch);
        assert!(writes[0].url.ends_with("/repos/o/r/pulls/7"));
        assert_eq!(body(&writes[0])["base"], "feat/base");
    }
}
