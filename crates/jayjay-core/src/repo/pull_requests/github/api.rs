use jayjay_network::{Fetch, HttpClient, Method, encode_path_segment};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use super::super::{ApiError, ForgeClient};
use crate::repo::forge_auth::TokenProvider;
use crate::repo::hosted_repo::{HostedRepo, RepoHost};

pub(crate) struct Api<F: Fetch = HttpClient, T: TokenProvider = RepoHost> {
    client: ForgeClient<F, T>,
    owner: String,
    repo: String,
}

impl Api {
    pub(crate) fn for_remote(remote: &HostedRepo) -> Self {
        Self::new(HttpClient::default(), RepoHost::GitHub, remote)
    }
}

impl<F: Fetch, T: TokenProvider> Api<F, T> {
    pub(crate) fn new(fetch: F, tokens: T, remote: &HostedRepo) -> Self {
        Self {
            client: ForgeClient::github(fetch, tokens),
            owner: remote.owner.clone(),
            repo: remote.repo.clone(),
        }
    }

    pub(crate) fn client(&self) -> &ForgeClient<F, T> {
        &self.client
    }

    pub(crate) fn owner(&self) -> &str {
        &self.owner
    }

    pub(crate) fn repo_path(&self, path: &str) -> String {
        format!(
            "repos/{}/{}/{path}",
            encode_path_segment(&self.owner),
            encode_path_segment(&self.repo)
        )
    }

    /// Binds `$owner` and `$repo` alongside `variables`.
    pub(crate) fn graphql<R: DeserializeOwned>(
        &self,
        query: &str,
        mut variables: Value,
    ) -> Result<R, ApiError> {
        variables["owner"] = json!(self.owner);
        variables["repo"] = json!(self.repo);
        let body = json!({ "query": query, "variables": variables });
        let response: GraphQlResponse<R> = self.client.send(Method::Post, "graphql", &body)?;
        if let Some(error) = response.errors.into_iter().next() {
            let status = (error.kind.as_deref() == Some("NOT_FOUND")).then_some(404);
            return Err(ApiError::new(status, error.message));
        }
        response
            .data
            .ok_or_else(|| ApiError::new(None, "GitHub returned no data"))
    }
}

#[derive(Deserialize)]
struct GraphQlResponse<R> {
    data: Option<R>,
    #[serde(default)]
    errors: Vec<GraphQlError>,
}

#[derive(Deserialize)]
struct GraphQlError {
    message: String,
    #[serde(rename = "type")]
    kind: Option<String>,
}
