use std::fmt;

use jayjay_network::{NetError, Response};
use serde::Deserialize;
use serde_json::Value;

use crate::repo::hosted_repo::RepoHost;

#[derive(Debug)]
pub(crate) struct ApiError {
    status: Option<u16>,
    message: String,
}

/// GitHub puts the reason in `errors[].message`; GitLab's `message` may be a list or a field map.
#[derive(Deserialize)]
struct ErrorBody {
    #[serde(default)]
    message: Value,
    #[serde(default)]
    errors: Vec<Value>,
}

impl ErrorBody {
    fn text(&self) -> String {
        let details = self
            .errors
            .iter()
            .filter_map(|error| error.get("message"))
            .map(flatten)
            .filter(|detail| !detail.is_empty())
            .collect::<Vec<_>>();
        [flatten(&self.message), details.join("; ")]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(": ")
    }
}

fn flatten(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        Value::Array(items) => items.iter().map(flatten).collect::<Vec<_>>().join("; "),
        Value::Object(fields) => fields
            .iter()
            .map(|(field, detail)| format!("{field} {}", flatten(detail)))
            .collect::<Vec<_>>()
            .join("; "),
        other => other.to_string(),
    }
}

impl ApiError {
    pub(crate) fn new(status: Option<u16>, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub(crate) fn status(&self) -> Option<u16> {
        self.status
    }

    pub(crate) fn from_response(response: &Response) -> Self {
        let message = serde_json::from_slice::<ErrorBody>(&response.body)
            .map(|body| body.text())
            .unwrap_or_default();
        Self {
            status: Some(response.status),
            message,
        }
    }

    pub(crate) fn unexpected(response: &Response, host: RepoHost) -> Self {
        Self {
            status: Some(response.status),
            message: format!("unexpected response from {}", host.display_name()),
        }
    }
}

impl From<NetError> for ApiError {
    fn from(error: NetError) -> Self {
        Self {
            status: None,
            message: error.to_string(),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.status {
            None => return f.write_str(&self.message),
            Some(401) => f.write_str("unauthorized (401)")?,
            Some(403) => f.write_str("forbidden (403)")?,
            Some(404) => f.write_str("not found (404)")?,
            Some(422) => f.write_str("invalid data (422)")?,
            Some(other) => write!(f, "HTTP {other}")?,
        }
        if self.message.is_empty() {
            Ok(())
        } else {
            write!(f, ": {}", self.message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error(status: u16, body: &str) -> String {
        ApiError::from_response(&Response::new(status, body)).to_string()
    }

    #[test]
    fn keeps_the_forge_reason_from_every_error_body_shape() {
        assert_eq!(
            error(
                422,
                r#"{"message":"Validation Failed","errors":[{"resource":"PullRequest","code":"custom","message":"A pull request already exists for o:feat."}]}"#
            ),
            "invalid data (422): Validation Failed: A pull request already exists for o:feat."
        );
        assert_eq!(
            error(
                409,
                r#"{"message":["Another open merge request already exists for this source branch: !7"]}"#
            ),
            "HTTP 409: Another open merge request already exists for this source branch: !7"
        );
        assert_eq!(
            error(400, r#"{"message":{"target_branch":["does not exist"]}}"#),
            "HTTP 400: target_branch does not exist"
        );
        assert_eq!(error(502, "<html>"), "HTTP 502");
    }
}
