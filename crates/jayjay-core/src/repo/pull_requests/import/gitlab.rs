use super::super::gitlab::api::{GitLabApi, Transport};
use super::super::gitlab::{self, GitLabMrResponse, GitLabProjectResponse, MrHeadProject};
use super::plan::{ForkRepo, PrHeadRepo, ResolvedPullRequest};
use super::url::ParsedPullRequestUrl;
use crate::repo::Repo;
use crate::types::{ChecksStatus, JayError, JayResult};

const NOT_FOUND: &str = "GitLab returned not found for this merge request. A private merge request needs a login: run `glab auth login`, or set GITLAB_TOKEN when glab is not installed.";
const UNUSABLE_HEAD: &str =
    "GitLab did not return a usable head for this merge request; the fork may have been deleted.";

pub(super) fn resolve(
    repo: &Repo,
    parsed: &ParsedPullRequestUrl,
    ssh: bool,
) -> JayResult<ResolvedPullRequest> {
    resolve_with(parsed, ssh, Transport::for_repo(repo))
}

fn resolve_with(
    parsed: &ParsedPullRequestUrl,
    ssh: bool,
    mut api: impl GitLabApi,
) -> JayResult<ResolvedPullRequest> {
    let mr: GitLabMrResponse = api
        .get_json(&gitlab::merge_request_path(&parsed.base, parsed.number))
        .map_err(|error| error.into_core_error("the merge request", NOT_FOUND))?;
    let head = match mr.head_project() {
        Some(MrHeadProject::Same) => PrHeadRepo::SameRepository,
        Some(MrHeadProject::Fork(project_id)) => {
            let project: GitLabProjectResponse = api
                .get_json(&gitlab::project_path(project_id))
                // A deleted fork 404s while the merge request that references it lives on.
                .map_err(|error| {
                    error.into_core_error("the merge request's source project", UNUSABLE_HEAD)
                })?;
            PrHeadRepo::Fork(ForkRepo::from_gitlab(&project, ssh)?)
        }
        None => return Err(JayError::internal(UNUSABLE_HEAD)),
    };
    Ok(ResolvedPullRequest::from_gitlab(mr, head))
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
    use super::super::super::gitlab::api::ApiError;
    use super::*;
    use crate::repo::hosted_repo::{HostedRepo, RepoHost};

    const HEAD: &str = "3a1df4445c8fcf392a378bd54d52511d4bb2d461";

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

    fn project_json(http_url: &str, ssh_url: &str, namespace: &str) -> String {
        format!(
            r#"{{"http_url_to_repo": "{http_url}", "ssh_url_to_repo": "{ssh_url}",
                "path_with_namespace": "{namespace}"}}"#
        )
    }

    fn resolve(
        mr: &str,
        project: Option<&str>,
        ssh: bool,
    ) -> (JayResult<ResolvedPullRequest>, Vec<String>) {
        let project = project.map(str::to_owned);
        let mut paths = Vec::new();
        let result = resolve_with(&parsed(), ssh, |path: &str| {
            paths.push(path.to_owned());
            if path == "projects/group%2Fsub%2Fbase/merge_requests/45" {
                Ok(mr.to_owned())
            } else if path == "projects/7765" {
                project.clone().ok_or(ApiError::NotFound)
            } else {
                Err(ApiError::Propagate(JayError::internal("unexpected path")))
            }
        });
        (result, paths)
    }

    #[test]
    fn same_project_mr_resolves_without_a_project_request() {
        let (resolved, paths) = resolve(&mr_json(83542242, 83542242), None, false);
        let resolved = resolved.expect("same-project head");
        assert_eq!(resolved.head_branch, "feat/x");
        assert_eq!(resolved.head_commit_id, HEAD);
        assert_eq!(resolved.head, PrHeadRepo::SameRepository);
        assert_eq!(paths, vec!["projects/group%2Fsub%2Fbase/merge_requests/45"]);
    }

    #[test]
    fn fork_mr_takes_the_fork_clone_url_in_the_origin_scheme_and_its_namespace() {
        let project = project_json(
            "https://gitlab.com/gitlab-community/gitlab-org/base.git",
            "git@gitlab.com:gitlab-community/gitlab-org/base.git",
            "gitlab-community/gitlab-org/base",
        );
        let (resolved, paths) = resolve(&mr_json(7765, 83542242), Some(&project), true);
        assert_eq!(
            resolved.expect("fork head").head,
            PrHeadRepo::Fork(ForkRepo {
                name_hint: "gitlab-community/gitlab-org".into(),
                clone_url: "git@gitlab.com:gitlab-community/gitlab-org/base.git".into(),
            })
        );
        assert_eq!(paths[1], "projects/7765");
    }

    #[test]
    fn a_missing_merge_request_is_reported_with_the_login_hint() {
        let error = resolve_with(&parsed(), false, |_: &str| Err(ApiError::NotFound))
            .expect_err("a missing MR must fail");
        let message = error.to_string();
        assert!(message.contains("glab auth login"), "{message}");
        assert!(message.contains("GITLAB_TOKEN"), "{message}");
    }

    #[test]
    fn a_deleted_fork_project_or_missing_ids_is_an_unusable_head() {
        let (resolved, _) = resolve(&mr_json(7765, 83542242), None, false);
        let error = resolved.expect_err("deleted fork must fail");
        assert!(
            error.to_string().contains("fork may have been deleted"),
            "{error}"
        );

        let no_ids = r#"{"iid":45,"state":"opened","title":"t","web_url":"u","source_branch":"b","sha":"s"}"#;
        let (resolved, _) = resolve(no_ids, Some("{}"), false);
        let error = resolved.expect_err("missing ids must fail");
        assert!(
            error.to_string().contains("fork may have been deleted"),
            "{error}"
        );
    }

    #[test]
    fn a_transient_project_failure_is_a_load_error_not_a_deleted_fork() {
        let mr = mr_json(7765, 83542242);
        let error = resolve_with(&parsed(), false, |path: &str| {
            if path == "projects/group%2Fsub%2Fbase/merge_requests/45" {
                Ok(mr.clone())
            } else {
                Err(ApiError::Load("HTTP 429".into()))
            }
        })
        .expect_err("a transient failure must fail");
        let message = error.to_string();
        assert!(message.contains("source project"), "{message}");
        assert!(message.contains("HTTP 429"), "{message}");
        assert!(!message.contains("fork may have been deleted"), "{message}");
    }

    #[test]
    fn a_canceled_api_request_surfaces_as_canceled() {
        let error = resolve_with(&parsed(), false, |_: &str| {
            Err(ApiError::Propagate(JayError::Canceled))
        })
        .expect_err("canceled must propagate");
        assert!(matches!(error, JayError::Canceled), "{error}");
    }
}
