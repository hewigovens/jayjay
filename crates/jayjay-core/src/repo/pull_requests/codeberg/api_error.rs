use std::fmt;

use jayjay_network::{NetError, Response};
use serde::Deserialize;

#[derive(Debug)]
pub(crate) struct ApiError {
    status: Option<u16>,
    message: String,
}

#[derive(Deserialize)]
struct ErrorBody {
    #[serde(default)]
    message: String,
}

impl ApiError {
    pub(crate) fn status(&self) -> Option<u16> {
        self.status
    }

    pub(super) fn from_response(response: &Response) -> Self {
        let message = serde_json::from_slice::<ErrorBody>(&response.body)
            .map(|body| body.message)
            .unwrap_or_default();
        Self {
            status: Some(response.status),
            message,
        }
    }

    pub(super) fn unexpected(response: &Response) -> Self {
        Self {
            status: Some(response.status),
            message: "unexpected response from Codeberg".to_owned(),
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
        let cause = match self.status {
            None => return f.write_str(&self.message),
            Some(401) => "unauthorized (401)",
            Some(403) => "forbidden (403)",
            Some(404) => "not found (404)",
            Some(422) => "invalid data (422)",
            Some(other) => return write!(f, "HTTP {other}: {}", self.message),
        };
        if self.message.is_empty() {
            f.write_str(cause)
        } else {
            write!(f, "{cause}: {}", self.message)
        }
    }
}
