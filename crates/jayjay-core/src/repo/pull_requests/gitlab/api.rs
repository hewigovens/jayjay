use jayjay_network::{Fetch, HttpClient, NetError};
use serde::de::DeserializeOwned;

use super::client::fetch_text;
use crate::repo::Repo;
use crate::repo::environment::{find_existing_binary, glab_binary};
use crate::types::JayError;

// glab when installed: it refreshes OAuth tokens and reads the keyring, which a plain request from a Finder launch cannot.
pub(crate) enum Transport<'a, F: Fetch = HttpClient> {
    Glab(&'a Repo, F),
    Plain(F),
}

impl<'a> Transport<'a> {
    pub(crate) fn for_repo(repo: &'a Repo) -> Self {
        if find_existing_binary("glab").is_some() {
            Self::Glab(repo, HttpClient::default())
        } else {
            Self::Plain(HttpClient::default())
        }
    }
}

impl<F: Fetch> Transport<'_, F> {
    pub(crate) fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        serde_json::from_str(&self.get(path)?)
            .map_err(|_| ApiError::Load("the answer was not valid JSON".into()))
    }

    fn get(&self, path: &str) -> Result<String, ApiError> {
        match self {
            Self::Glab(repo, fetch) => via_glab(repo, fetch, path),
            Self::Plain(fetch) => via_plain_request(fetch, path),
        }
    }
}

fn via_glab(repo: &Repo, fetch: &impl Fetch, path: &str) -> Result<String, ApiError> {
    let output = repo
        .cancellable_output(
            &glab_binary(),
            &["api", "--hostname", "gitlab.com", path],
            "glab api",
        )
        .map_err(ApiError::Propagate)?;
    if output.status.success() {
        return Ok(Repo::stdout_text(&output));
    }
    let detail = Repo::stderr_text(&output);
    match GlabFailure::from_stderr(&detail) {
        Some(GlabFailure::NotFound) => Err(ApiError::NotFound),
        // A rejected glab login 401s even public projects.
        Some(GlabFailure::Unauthorized) => via_plain_request(fetch, path),
        None => Err(ApiError::Load(detail)),
    }
}

fn via_plain_request(fetch: &impl Fetch, path: &str) -> Result<String, ApiError> {
    fetch_text(fetch, path).map_err(|error| match error {
        NetError::NotFound => ApiError::NotFound,
        error => ApiError::Load(error.to_string()),
    })
}

pub(crate) enum ApiError {
    NotFound,
    Load(String),
    Propagate(JayError),
}

impl ApiError {
    pub(crate) fn into_core_error(self, subject: &str, not_found: &str) -> JayError {
        match self {
            Self::NotFound => JayError::internal(not_found),
            Self::Load(detail) => {
                JayError::internal(format!("Couldn't load {subject} from GitLab: {detail}"))
            }
            Self::Propagate(error) => error,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum GlabFailure {
    NotFound,
    Unauthorized,
}

impl GlabFailure {
    fn from_stderr(detail: &str) -> Option<Self> {
        if detail.contains("(HTTP 404)") {
            Some(Self::NotFound)
        } else if detail.contains("(HTTP 401)") {
            Some(Self::Unauthorized)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_a_failed_glab_request_from_its_stderr() {
        assert_eq!(
            GlabFailure::from_stderr("glab: 404 Project Not Found (HTTP 404)"),
            Some(GlabFailure::NotFound)
        );
        assert_eq!(
            GlabFailure::from_stderr("glab: 401 Unauthorized (HTTP 401)"),
            Some(GlabFailure::Unauthorized)
        );
        for other in [
            "glab: 500 Internal Server Error (HTTP 500)",
            "Get \"https://gitlab.com/api/v4/projects/87140412\": context deadline exceeded",
        ] {
            assert_eq!(GlabFailure::from_stderr(other), None, "{other}");
        }
    }
}
