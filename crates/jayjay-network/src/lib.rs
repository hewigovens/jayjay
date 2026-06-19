//! Shared blocking HTTP helpers for JayJay.
//!
//! One configured `ureq` agent so every caller shares the same timeout and
//! redirect policy. Calls block; run them on a background executor.

use std::fmt;
use std::io::Read;
use std::sync::LazyLock;
use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// Body cap for text responses (JSON APIs): 2 MiB.
const DEFAULT_TEXT_CAP: u64 = 2 * 1024 * 1024;

static AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| {
    ureq::Agent::config_builder()
        .timeout_global(Some(REQUEST_TIMEOUT))
        .build()
        .into()
});

/// Why a GET did not yield a usable body. A 404 ("no such resource") must stay
/// distinct from a transport failure, which must never read as an empty result.
#[derive(Debug, PartialEq, Eq)]
pub enum NetError {
    /// The server returned HTTP 404.
    NotFound,
    /// Any other non-success status (e.g. 401/403/429/5xx).
    Http(u16),
    /// Transport error, timeout, oversize body, or invalid UTF-8.
    Transport,
}

impl fmt::Display for NetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetError::NotFound => write!(f, "not found"),
            NetError::Http(status) => write!(f, "HTTP {status}"),
            NetError::Transport => write!(f, "transport error"),
        }
    }
}

/// An optional bearer/token header to send with a request.
#[derive(Default)]
pub struct Auth(Option<String>);

impl Auth {
    /// `Authorization: token <value>` (Forgejo/Gitea style) when `value` is set.
    pub fn token(value: Option<String>) -> Self {
        Auth(
            value
                .filter(|v| !v.is_empty())
                .map(|v| format!("token {v}")),
        )
    }

    /// `Authorization: Bearer <value>` (GitLab / OAuth style) when `value` is set.
    pub fn bearer(value: Option<String>) -> Self {
        Auth(
            value
                .filter(|v| !v.is_empty())
                .map(|v| format!("Bearer {v}")),
        )
    }
}

/// GET `url` as text. See [`NetError`] for the failure cases.
pub fn get_text(url: &str) -> Result<String, NetError> {
    get_text_with_auth(url, &Auth::default())
}

/// GET `url` as text with an optional `Authorization` header.
pub fn get_text_with_auth(url: &str, auth: &Auth) -> Result<String, NetError> {
    let bytes = get_bytes(url, DEFAULT_TEXT_CAP, auth)?;
    String::from_utf8(bytes).map_err(|_| NetError::Transport)
}

/// GET `url`, returning up to `max_bytes` of the body.
pub fn get_bytes(url: &str, max_bytes: u64, auth: &Auth) -> Result<Vec<u8>, NetError> {
    let mut request = AGENT.get(url);
    if let Some(header) = &auth.0 {
        request = request.header("Authorization", header);
    }
    let mut response = request.call().map_err(|_| NetError::Transport)?;
    let status = response.status().as_u16();
    if status != 200 {
        return Err(match status {
            404 => NetError::NotFound,
            other => NetError::Http(other),
        });
    }
    let mut bytes = Vec::new();
    response
        .body_mut()
        .as_reader()
        .take(max_bytes)
        .read_to_end(&mut bytes)
        .map_err(|_| NetError::Transport)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_auth_omits_empty_and_missing() {
        assert!(Auth::token(None).0.is_none());
        assert!(Auth::token(Some(String::new())).0.is_none());
        assert_eq!(
            Auth::token(Some("abc".into())).0.as_deref(),
            Some("token abc")
        );
    }

    #[test]
    fn bearer_auth_omits_empty_and_missing() {
        assert!(Auth::bearer(None).0.is_none());
        assert!(Auth::bearer(Some(String::new())).0.is_none());
        assert_eq!(
            Auth::bearer(Some("abc".into())).0.as_deref(),
            Some("Bearer abc")
        );
    }

    #[test]
    fn net_error_distinguishes_not_found_from_other_http() {
        assert_eq!(NetError::NotFound, NetError::NotFound);
        assert_ne!(NetError::NotFound, NetError::Http(404));
        assert_ne!(NetError::Http(429), NetError::Transport);
    }
}
