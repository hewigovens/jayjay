use jayjay_network::{Fetch, Method, encode_path_segment};
use serde_json::json;

use super::forge::{ForgeTarget, created_with_number, failed, non_empty_or, updated};
use crate::repo::forge_auth::TokenProvider;
use crate::repo::hosted_repo::HostedRepo;
use crate::repo::pull_requests::gitlab::{
    GitLabMrResponse, MrHeadProject, merge_request_path, merge_requests_path,
};
use crate::repo::pull_requests::{ApiError, ForgeClient};
use crate::types::{ChecksStatus, SubmittedLayer};

pub(super) fn create_or_update_mr(
    api: &ForgeClient<impl Fetch, impl TokenProvider>,
    remote: &HostedRepo,
    target: &ForgeTarget,
) -> SubmittedLayer {
    // A closed MR's target can't be edited, and a fork's branch can share the name.
    let path = format!(
        "{}?source_branch={}&state=opened",
        merge_requests_path(remote),
        encode_path_segment(&target.bookmark)
    );
    match api.get::<Vec<GitLabMrResponse>>(&path) {
        Ok(open) => match open
            .into_iter()
            .find(|mr| matches!(mr.head_project(), Some(MrHeadProject::Same)))
        {
            Some(mr) => update_target(api, remote, target, mr),
            None => create_mr(api, remote, target),
        },
        Err(error) => failed(target, error.to_string()),
    }
}

fn update_target(
    api: &ForgeClient<impl Fetch, impl TokenProvider>,
    remote: &HostedRepo,
    target: &ForgeTarget,
    mr: GitLabMrResponse,
) -> SubmittedLayer {
    let mr = mr.into_pr_info(ChecksStatus::None);
    let title = non_empty_or(&target.title, &target.bookmark);
    let description = non_empty_or(&target.body, &title);
    let body = json!({
        "target_branch": target.base,
        "title": title,
        "description": description,
        // Enforce branch retention on re-submit so already-created MRs stay stack-safe.
        "remove_source_branch": false,
    });
    let result: Result<GitLabMrResponse, ApiError> =
        api.send(Method::Put, &merge_request_path(remote, mr.number), &body);
    match result {
        Ok(_) => updated(
            target,
            title,
            mr.number,
            mr.url,
            format!("MR !{}", mr.number),
        ),
        Err(error) => failed(target, error.to_string()),
    }
}

fn create_mr(
    api: &ForgeClient<impl Fetch, impl TokenProvider>,
    remote: &HostedRepo,
    target: &ForgeTarget,
) -> SubmittedLayer {
    let title = non_empty_or(&target.title, &target.bookmark);
    let description = non_empty_or(&target.body, &title);
    let body = json!({
        "source_branch": target.bookmark,
        "target_branch": target.base,
        "title": title,
        "description": description,
        // GitLab retargets the MR above only while the merged branch survives; deleting it would orphan the stack.
        "remove_source_branch": false,
    });
    match api.send::<GitLabMrResponse>(Method::Post, &merge_requests_path(remote), &body) {
        Ok(mr) => {
            let mr = mr.into_pr_info(ChecksStatus::None);
            created_with_number(target, title, mr.number, mr.url)
        }
        Err(error) => failed(target, error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use jayjay_network::{MockFetch, Request, Response};

    use super::*;
    use crate::repo::forge_auth::{ForgeToken, TokenSource};
    use crate::repo::hosted_repo::RepoHost;
    use crate::types::StackLayerOutcome;

    const MR: &str = r#"{"iid":7,"state":"opened","title":"Add x","web_url":"https://gitlab.com/g/r/-/merge_requests/7","source_project_id":1,"target_project_id":1}"#;

    fn submit(
        respond: impl Fn(&Request) -> Result<Response, jayjay_network::NetError>,
    ) -> (SubmittedLayer, Vec<Request>) {
        let fetch = MockFetch::new(respond);
        let token = Some(ForgeToken {
            secret: "t".into(),
            source: TokenSource::Environment("GITLAB_TOKEN"),
        });
        let api = ForgeClient::gitlab(&fetch, token);
        let remote = HostedRepo {
            host: RepoHost::GitLab,
            owner: "g".into(),
            repo: "r".into(),
        };
        let target = ForgeTarget {
            bookmark: "feat/x".into(),
            base: "main".into(),
            title: "Add x".into(),
            body: String::new(),
        };
        let layer = create_or_update_mr(&api, &remote, &target);
        (layer, fetch.writes())
    }

    fn body(request: &Request) -> serde_json::Value {
        serde_json::from_str(request.body_text()).unwrap()
    }

    #[test]
    fn creates_a_merge_request_that_keeps_its_source_branch() {
        let (layer, writes) = submit(|request| match request.method {
            Method::Get => Response::ok(200, "[]"),
            _ => Response::ok(201, MR),
        });
        assert_eq!(
            (layer.outcome, layer.pr_number),
            (StackLayerOutcome::Created, 7)
        );
        assert_eq!(
            writes[0].url,
            "https://gitlab.com/api/v4/projects/g%2Fr/merge_requests"
        );
        assert_eq!(
            body(&writes[0]),
            json!({
                "source_branch": "feat/x", "target_branch": "main", "title": "Add x",
                "description": "Add x", "remove_source_branch": false,
            })
        );
    }

    #[test]
    fn retargets_an_open_merge_request() {
        let (layer, writes) = submit(|request| match request.method {
            Method::Get => {
                assert!(
                    request
                        .url
                        .ends_with("?source_branch=feat%2Fx&state=opened")
                );
                Response::ok(200, format!("[{MR}]"))
            }
            _ => Response::ok(200, MR),
        });
        assert_eq!(layer.detail, "MR !7 → base main");
        assert_eq!(writes[0].method, Method::Put);
        assert!(writes[0].url.ends_with("/merge_requests/7"));
        assert_eq!(body(&writes[0])["remove_source_branch"], json!(false));
    }

    #[test]
    fn a_fork_merge_request_on_the_same_branch_name_is_left_alone() {
        let fork = MR.replace(r#""source_project_id":1"#, r#""source_project_id":9"#);
        let (layer, writes) = submit(move |request| match request.method {
            Method::Get => Response::ok(200, format!("[{fork}]")),
            _ => Response::ok(201, MR),
        });
        assert_eq!(layer.outcome, StackLayerOutcome::Created);
        assert_eq!(writes[0].method, Method::Post);
    }
}
