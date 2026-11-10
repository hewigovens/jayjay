use jayjay_network::graphql::{GraphQlRequest, GraphQlResponse, GraphQlResponseError};
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
        format!("{}/{path}", self.repository_path())
    }

    fn repository_path(&self) -> String {
        format!(
            "repos/{}/{}",
            encode_path_segment(&self.owner),
            encode_path_segment(&self.repo)
        )
    }

    /// The repository's default branch, or `None` when GitHub doesn't report one.
    pub(crate) fn default_branch(&self) -> Option<String> {
        #[derive(Deserialize)]
        struct Repository {
            #[serde(default)]
            default_branch: String,
        }
        let repository: Repository = self.client.get(&self.repository_path()).ok()?;
        Some(repository.default_branch).filter(|branch| !branch.is_empty())
    }

    /// Binds `$owner` and `$repo` alongside `variables`.
    pub(crate) fn graphql<R: DeserializeOwned>(
        &self,
        query: &str,
        mut variables: Value,
    ) -> Result<R, ApiError> {
        variables["owner"] = json!(self.owner);
        variables["repo"] = json!(self.repo);
        let body = GraphQlRequest { query, variables };
        let response: GraphQlResponse = self.client.send(Method::Post, "graphql", &body)?;
        response.into_data().map_err(|error| {
            let status = match &error {
                GraphQlResponseError::Errors(errors) => errors
                    .first()
                    .is_some_and(|error| error.kind() == Some("NOT_FOUND"))
                    .then_some(404),
                _ => None,
            };
            ApiError::new(status, error.to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use jayjay_network::{MockFetch, Response};

    use super::*;
    use crate::repo::forge_auth::{ForgeToken, TokenSource};

    #[test]
    fn default_branch_reads_the_repository() {
        for (status, body, expected) in [
            (200, r#"{"default_branch":"develop"}"#, Some("develop")),
            (200, r#"{"default_branch":""}"#, None),
            (404, r#"{"message":"Not Found"}"#, None),
        ] {
            let fetch = MockFetch::new(move |request| {
                assert!(
                    request.url.ends_with("/repos/owner/repo"),
                    "{}",
                    request.url
                );
                Response::ok(status, body)
            });
            let api = Api::new(
                fetch,
                Some(ForgeToken {
                    secret: "token".into(),
                    source: TokenSource::Stored,
                }),
                &HostedRepo {
                    host: RepoHost::GitHub,
                    owner: "owner".into(),
                    repo: "repo".into(),
                },
            );
            assert_eq!(api.default_branch().as_deref(), expected);
        }
    }

    #[test]
    fn graphql_errors_keep_their_status_when_partial_data_is_incompatible() {
        #[derive(Debug, Deserialize)]
        struct Data {
            _value: u32,
        }

        for (kind, status) in [("NOT_FOUND", Some(404)), ("FORBIDDEN", None)] {
            let body = json!({
                "data": { "_value": null },
                "errors": [
                    { "type": kind, "message": "first reason" },
                    { "type": "NOT_FOUND", "message": "second reason" }
                ]
            })
            .to_string();
            let fetch = MockFetch::new(|_| Response::ok(200, body.clone()));
            let api = Api::new(
                fetch,
                Some(ForgeToken {
                    secret: "token".into(),
                    source: TokenSource::Stored,
                }),
                &HostedRepo {
                    host: RepoHost::GitHub,
                    owner: "owner".into(),
                    repo: "repo".into(),
                },
            );
            let error = api
                .graphql::<Data>("query { _value }", json!({}))
                .unwrap_err();
            assert_eq!(error.status(), status);
            assert!(error.to_string().contains("first reason"));
            assert!(!error.to_string().contains("second reason"));
        }
    }
}
