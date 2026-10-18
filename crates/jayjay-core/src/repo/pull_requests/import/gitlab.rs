use jayjay_network::{Fetch, HttpClient};

use super::super::gitlab::{self, GitLabMrResponse, GitLabProjectResponse, MrHeadProject};
use super::super::{ApiError, ForgeClient};
use super::plan::{ForkRepo, PrHeadRepo, ResolvedPullRequest};
use super::url::ParsedPullRequestUrl;
use crate::repo::forge_auth::TokenProvider;
use crate::repo::hosted_repo::RepoHost;
use crate::types::{ChecksStatus, JayError, JayResult};

const UNUSABLE_HEAD: &str =
    "GitLab did not return a usable head for this merge request; the fork may have been deleted.";

pub(super) fn resolve(parsed: &ParsedPullRequestUrl, ssh: bool) -> JayResult<ResolvedPullRequest> {
    resolve_with(
        parsed,
        ssh,
        &ForgeClient::gitlab(HttpClient::default(), RepoHost::GitLab),
    )
}

fn resolve_with(
    parsed: &ParsedPullRequestUrl,
    ssh: bool,
    api: &ForgeClient<impl Fetch, impl TokenProvider>,
) -> JayResult<ResolvedPullRequest> {
    let mr: GitLabMrResponse = api
        .get(&gitlab::merge_request_path(&parsed.base, parsed.number))
        .map_err(|error| {
            let hint = RepoHost::GitLab.sign_in_hint();
            load_error(
                error,
                "the merge request",
                &format!("GitLab returned not found for this merge request. A private merge request needs a login: {hint}"),
            )
        })?;
    let head = match mr.head_project() {
        Some(MrHeadProject::Same) => PrHeadRepo::SameRepository,
        Some(MrHeadProject::Fork(project_id)) => {
            let project: GitLabProjectResponse = api
                .get(&gitlab::project_path(project_id))
                // A deleted fork 404s while the merge request that references it lives on.
                .map_err(|error| {
                    load_error(error, "the merge request's source project", UNUSABLE_HEAD)
                })?;
            PrHeadRepo::Fork(ForkRepo::from_gitlab(&project, ssh)?)
        }
        None => return Err(JayError::internal(UNUSABLE_HEAD)),
    };
    Ok(ResolvedPullRequest::from_gitlab(mr, head))
}

fn load_error(error: ApiError, subject: &str, not_found: &str) -> JayError {
    if error.status() == Some(404) {
        JayError::internal(not_found)
    } else {
        JayError::internal(format!("Couldn't load {subject} from GitLab: {error}"))
    }
}

impl ResolvedPullRequest {
    fn from_gitlab(mr: GitLabMrResponse, head: PrHeadRepo) -> Self {
        let head_branch = mr.head_branch().to_owned();
        let head_commit_id = mr.head_sha().unwrap_or_default().to_owned();
        let info = mr.into_pr_info(ChecksStatus::None);
        Self {
            number: info.number,
            title: info.title,
            state: info.state,
            url: info.url,
            head_branch,
            head_commit_id,
            head,
        }
    }
}

impl ForkRepo {
    fn from_gitlab(project: &GitLabProjectResponse, ssh: bool) -> JayResult<Self> {
        let Some(clone_url) = project.pinned_clone_url(ssh) else {
            return Err(JayError::internal(
                "The merge request's fork did not answer with a clone URL on gitlab.com, so it was not added as a remote.",
            ));
        };
        let Some(name_hint) = project.namespace() else {
            return Err(JayError::internal(UNUSABLE_HEAD));
        };
        Ok(Self {
            name_hint: name_hint.to_owned(),
            clone_url: clone_url.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use jayjay_network::{MockFetch, NetError, Request, Response};

    use super::*;
    use crate::repo::forge_auth::ForgeToken;
    use crate::repo::hosted_repo::{HostedRepo, RepoHost};

    const HEAD: &str = "3a1df4445c8fcf392a378bd54d52511d4bb2d461";
    const MR_URL: &str = "https://gitlab.com/api/v4/projects/group%2Fsub%2Fbase/merge_requests/45";
    const FORK_URL: &str = "https://gitlab.com/api/v4/projects/7765";

    fn parsed() -> ParsedPullRequestUrl {
        ParsedPullRequestUrl {
            base: HostedRepo {
                host: RepoHost::GitLab,
                owner: "group/sub".into(),
                repo: "base".into(),
            },
            number: 45,
        }
    }

    fn mr_json(source: u64, target: u64) -> String {
        format!(
            r#"{{
                "iid": 45, "state": "opened", "title": "feat: fork it",
                "web_url": "https://gitlab.com/group/sub/base/-/merge_requests/45",
                "source_branch": "feat/x", "sha": "{HEAD}",
                "source_project_id": {source}, "target_project_id": {target}
            }}"#
        )
    }

    fn resolve(
        ssh: bool,
        respond: impl Fn(&Request) -> Result<Response, NetError>,
    ) -> (JayResult<ResolvedPullRequest>, Vec<String>) {
        let fetch = MockFetch::new(respond);
        let api = ForgeClient::gitlab(&fetch, None::<ForgeToken>);
        let result = resolve_with(&parsed(), ssh, &api);
        let urls = fetch
            .requests()
            .iter()
            .map(|request| request.url.clone())
            .collect();
        (result, urls)
    }

    fn mr_then(
        project: Result<Response, NetError>,
    ) -> impl Fn(&Request) -> Result<Response, NetError> {
        move |request| {
            if request.url == MR_URL {
                Response::ok(200, mr_json(7765, 83542242))
            } else {
                project.clone()
            }
        }
    }

    #[test]
    fn same_project_mr_resolves_without_a_project_request() {
        let (resolved, urls) = resolve(false, |_| Response::ok(200, mr_json(83542242, 83542242)));
        let resolved = resolved.expect("same-project head");
        assert_eq!(resolved.head_branch, "feat/x");
        assert_eq!(resolved.head_commit_id, HEAD);
        assert_eq!(resolved.head, PrHeadRepo::SameRepository);
        assert_eq!(urls, [MR_URL]);
    }

    #[test]
    fn fork_mr_takes_the_fork_clone_url_in_the_origin_scheme_and_its_namespace() {
        let project = r#"{"http_url_to_repo": "https://gitlab.com/gitlab-community/gitlab-org/base.git",
            "ssh_url_to_repo": "git@gitlab.com:gitlab-community/gitlab-org/base.git",
            "path_with_namespace": "gitlab-community/gitlab-org/base"}"#;
        let (resolved, urls) = resolve(true, mr_then(Response::ok(200, project)));
        assert_eq!(
            resolved.expect("fork head").head,
            PrHeadRepo::Fork(ForkRepo {
                name_hint: "gitlab-community/gitlab-org".into(),
                clone_url: "git@gitlab.com:gitlab-community/gitlab-org/base.git".into(),
            })
        );
        assert_eq!(urls, [MR_URL, FORK_URL]);
    }

    #[test]
    fn a_missing_merge_request_is_reported_with_the_login_hint() {
        let (resolved, _) = resolve(false, |_| Response::ok(404, ""));
        let message = resolved.expect_err("a missing MR must fail").to_string();
        assert!(
            message.contains("glab auth login") && message.contains("GITLAB_TOKEN"),
            "{message}"
        );
    }

    #[test]
    fn a_deleted_fork_or_missing_ids_is_an_unusable_head_but_a_transient_failure_is_not() {
        let (resolved, _) = resolve(false, mr_then(Response::ok(404, "")));
        let error = resolved.expect_err("deleted fork must fail").to_string();
        assert!(error.contains("fork may have been deleted"), "{error}");

        let no_ids = r#"{"iid":45,"state":"opened","title":"t","web_url":"u","source_branch":"b","sha":"s"}"#;
        let (resolved, _) = resolve(false, move |_| Response::ok(200, no_ids));
        let error = resolved.expect_err("missing ids must fail").to_string();
        assert!(error.contains("fork may have been deleted"), "{error}");

        let (resolved, _) = resolve(false, mr_then(Response::ok(429, "")));
        let error = resolved
            .expect_err("a transient failure must fail")
            .to_string();
        assert!(
            error.contains("source project") && error.contains("HTTP 429"),
            "{error}"
        );
        assert!(!error.contains("fork may have been deleted"), "{error}");
    }
}
