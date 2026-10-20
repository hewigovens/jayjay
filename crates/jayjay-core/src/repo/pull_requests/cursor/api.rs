use jayjay_network::{Fetch, HttpClient, Method, encode_path_segment};
use serde::Deserialize;
use serde::de::IgnoredAny;

use super::super::{ApiError, ForgeClient};
use super::pull_request::{CreatePull, OriginPull, UpdatePull};
use super::repo::{OriginRepo, create_failure_message};
use super::status::OriginCheckRun;
use crate::repo::forge_auth::TokenProvider;
use crate::repo::hosted_repo::{HostedRepo, RepoHost};

pub(crate) struct Api<F: Fetch = HttpClient, T: TokenProvider = RepoHost> {
    client: ForgeClient<F, T>,
    repo_path: String,
}

impl Api {
    pub(crate) fn for_remote(remote: &HostedRepo) -> Self {
        Self::new(HttpClient::default(), RepoHost::Cursor, remote)
    }
}

impl<F: Fetch, T: TokenProvider> Api<F, T> {
    pub(crate) fn new(fetch: F, tokens: T, remote: &HostedRepo) -> Self {
        Self {
            client: ForgeClient::cursor(fetch, tokens),
            repo_path: format!(
                "repos/{}/{}",
                encode_path_segment(&remote.owner),
                encode_path_segment(&remote.repo)
            ),
        }
    }

    pub(crate) fn client(&self) -> &ForgeClient<F, T> {
        &self.client
    }

    /// The default branch, unless the repository is an inbound GitHub mirror, which can't host pull requests.
    pub(crate) fn pull_request_base(&self) -> Result<String, String> {
        let repository: OriginRepo = self
            .client
            .get(&self.repo_path)
            .map_err(|error| error.to_string())?;
        repository.pull_request_base()
    }

    pub(super) fn pulls(&self, head: &str) -> Result<Vec<OriginPull>, ApiError> {
        let path = format!(
            "{}/pulls?head={}&state=all&pageSize=30",
            self.repo_path,
            encode_path_segment(head)
        );
        self.client
            .get::<PullList>(&path)
            .map(|list| list.pull_requests)
    }

    pub(crate) fn create_pull(
        &self,
        head: &str,
        base: &str,
        title: &str,
        body: &str,
    ) -> Result<OriginPull, String> {
        let path = format!("{}/pulls", self.repo_path);
        let pull = CreatePull {
            title,
            body,
            head,
            base,
        };
        self.client
            .send(Method::Post, &path, &pull)
            .map_err(|error| create_failure_message(&error))
    }

    pub(crate) fn update_pull(&self, number: u32, pull: &UpdatePull) -> Result<(), ApiError> {
        let path = format!("{}/pulls/{number}", self.repo_path);
        self.client
            .send::<IgnoredAny>(Method::Patch, &path, pull)
            .map(drop)
    }

    pub(super) fn check_runs(&self, sha: &str) -> Result<Vec<OriginCheckRun>, ApiError> {
        let path = format!(
            "{}/commits/{}/check-runs?pageSize=100",
            self.repo_path,
            encode_path_segment(sha)
        );
        let mut page_path = path.clone();
        let mut runs = Vec::new();
        loop {
            let page: CheckRunList = self.client.get(&page_path)?;
            runs.extend(page.check_runs);
            if page.next_page_token.is_empty() {
                return Ok(runs);
            }
            page_path = format!(
                "{path}&pageToken={}",
                encode_path_segment(&page.next_page_token)
            );
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PullList {
    #[serde(default)]
    pull_requests: Vec<OriginPull>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckRunList {
    #[serde(default)]
    check_runs: Vec<OriginCheckRun>,
    #[serde(default)]
    next_page_token: String,
}
