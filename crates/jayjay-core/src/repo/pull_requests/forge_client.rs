use std::borrow::Cow;

use jayjay_network::{Auth, DEFAULT_TEXT_CAP, Fetch, Header, HttpClient, Method, Request};
use serde::Serialize;
use serde::de::{DeserializeOwned, IgnoredAny};

use super::ApiError;
use crate::repo::forge_auth::TokenProvider;
use crate::repo::hosted_repo::RepoHost;
use crate::types::{JayError, JayResult};

pub(crate) struct ForgeClient<F: Fetch = HttpClient, T: TokenProvider = RepoHost> {
    fetch: F,
    tokens: T,
    host: RepoHost,
    root: Cow<'static, str>,
    headers: &'static [(&'static str, &'static str)],
}

impl<F: Fetch, T: TokenProvider> ForgeClient<F, T> {
    pub(crate) fn github(fetch: F, tokens: T) -> Self {
        Self {
            fetch,
            tokens,
            host: RepoHost::GitHub,
            root: Self::github_root(),
            headers: &[
                ("Accept", "application/vnd.github+json"),
                ("X-GitHub-Api-Version", "2022-11-28"),
            ],
        }
    }

    pub(crate) fn gitlab(fetch: F, tokens: T) -> Self {
        Self {
            fetch,
            tokens,
            host: RepoHost::GitLab,
            root: "https://gitlab.com/api/v4".into(),
            headers: &[],
        }
    }

    // Lets the feature videos record a stacked-PR submit against a local stub; release builds always talk to GitHub.
    fn github_root() -> Cow<'static, str> {
        #[cfg(debug_assertions)]
        if let Ok(root) = std::env::var("JAYJAY_GITHUB_API_ROOT") {
            return root.into();
        }
        "https://api.github.com".into()
    }

    pub(crate) fn has_token(&self) -> bool {
        self.tokens.token().is_some()
    }

    pub(crate) fn preflight(&self) -> JayResult<()> {
        let name = self.host.display_name();
        let hint = self.host.sign_in_hint();
        self.tokens.forget();
        if !self.has_token() {
            return Err(JayError::internal(format!(
                "This needs a {name} login. {hint}"
            )));
        }
        self.get::<IgnoredAny>("user").map(drop).map_err(|error| {
            JayError::internal(format!(
                "{name} didn't accept the current login: {error}. {hint}"
            ))
        })
    }

    pub(crate) fn get<R: DeserializeOwned>(&self, path: &str) -> Result<R, ApiError> {
        let url = self.url(path);
        self.call(|auth| Request::get(url.clone(), DEFAULT_TEXT_CAP, auth))
    }

    pub(crate) fn send<R: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: &impl Serialize,
    ) -> Result<R, ApiError> {
        let url = self.url(path);
        self.call(|auth| {
            Request::json(url.clone(), method, auth, body)
                .expect("forge request bodies are plain structs")
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{path}", self.root)
    }

    fn call<R: DeserializeOwned>(&self, request: impl Fn(&Auth) -> Request) -> Result<R, ApiError> {
        let headers = self.headers;
        let response = self
            .tokens
            .fetch(&self.fetch, |auth| {
                let mut request = request(auth);
                request.headers.extend(
                    headers
                        .iter()
                        .map(|(name, value)| Header::new(*name, *value)),
                );
                request
            })?
            .error_for_status()
            .map_err(|response| ApiError::from_response(&response))?;
        response
            .json()
            .map_err(|_| ApiError::unexpected(&response, self.host))
    }
}
