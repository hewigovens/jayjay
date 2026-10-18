use std::fmt;

use crate::encode_path_segment;

pub const DEFAULT_TEXT_CAP: u32 = 2 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub name: String,
    pub value: String,
}

impl Header {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub url: String,
    pub method: Method,
    pub headers: Vec<Header>,
    pub body: Option<Vec<u8>>,
    pub max_response_bytes: u32,
}

impl Request {
    pub fn new(
        url: impl Into<String>,
        method: Method,
        headers: Vec<Header>,
        body: Option<Vec<u8>>,
        max_response_bytes: u32,
    ) -> Self {
        Self {
            url: url.into(),
            method,
            headers,
            body,
            max_response_bytes,
        }
    }

    pub fn get(url: impl Into<String>, max_response_bytes: u32, auth: &Auth) -> Self {
        Self::new(
            url,
            Method::Get,
            auth.header().into_iter().collect(),
            None,
            max_response_bytes,
        )
    }

    /// A form-encoded POST that asks for a JSON answer, as OAuth token endpoints expect.
    pub fn form(url: impl Into<String>, fields: &[(&str, &str)]) -> Self {
        let body = fields
            .iter()
            .map(|(name, value)| {
                format!(
                    "{}={}",
                    encode_path_segment(name),
                    encode_path_segment(value)
                )
            })
            .collect::<Vec<_>>()
            .join("&");
        Self::new(
            url,
            Method::Post,
            vec![
                Header::new("Content-Type", "application/x-www-form-urlencoded"),
                Header::new("Accept", "application/json"),
            ],
            Some(body.into_bytes()),
            DEFAULT_TEXT_CAP,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status: u16, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: body.into(),
        }
    }

    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// The failed response is returned whole, so a client can read the server's error body.
    pub fn error_for_status(self) -> Result<Self, Self> {
        if self.is_success() {
            Ok(self)
        } else {
            Err(self)
        }
    }

    pub fn into_bytes(mut self, max_bytes: u32) -> Result<Vec<u8>, NetError> {
        if !self.is_success() {
            return Err(match self.status {
                404 => NetError::NotFound,
                other => NetError::Http(other),
            });
        }

        self.body.truncate(max_bytes as usize);
        Ok(self.body)
    }

    pub fn into_text(self, max_bytes: u32) -> Result<String, NetError> {
        String::from_utf8(self.into_bytes(max_bytes)?).map_err(|_| NetError::Transport)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetError {
    NotFound,
    Http(u16),
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Auth(Option<String>);

impl Auth {
    fn scheme(value: Option<String>, scheme: &str) -> Self {
        Auth(
            value
                .filter(|value| !value.is_empty())
                .map(|value| format!("{scheme} {value}")),
        )
    }

    pub(crate) fn header(&self) -> Option<Header> {
        self.0
            .as_ref()
            .map(|value| Header::new("Authorization", value))
    }

    pub fn authorization(value: Option<String>) -> Self {
        Auth(value.filter(|value| !value.is_empty()))
    }

    pub fn token(value: Option<String>) -> Self {
        Self::scheme(value, "token")
    }

    pub fn bearer(value: Option<String>) -> Self {
        Self::scheme(value, "Bearer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_carries_auth_as_a_regular_header() {
        let request = Request::get(
            "https://example.test",
            10,
            &Auth::bearer(Some("abc".into())),
        );
        assert_eq!(
            request.headers,
            vec![Header::new("Authorization", "Bearer abc")]
        );
    }

    #[test]
    fn form_bodies_escape_every_reserved_character() {
        let request = Request::form(
            "https://example.test",
            &[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("scope", "repo read:org"),
            ],
        );
        assert_eq!(
            String::from_utf8(request.body.unwrap()).unwrap(),
            "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Adevice_code&scope=repo%20read%3Aorg"
        );
    }

    #[test]
    fn empty_and_missing_auth_are_omitted() {
        assert!(
            Request::get("https://example.test", 10, &Auth::token(None))
                .headers
                .is_empty()
        );
        assert!(
            Request::get(
                "https://example.test",
                10,
                &Auth::bearer(Some(String::new()))
            )
            .headers
            .is_empty()
        );
    }

    #[test]
    fn response_validation_distinguishes_status_and_transport_failures() {
        assert_eq!(
            Response {
                status: 404,
                headers: vec![],
                body: vec![],
            }
            .into_bytes(10),
            Err(NetError::NotFound)
        );
        assert_eq!(
            Response {
                status: 429,
                headers: vec![],
                body: vec![],
            }
            .into_bytes(10),
            Err(NetError::Http(429))
        );
        assert_eq!(
            Response {
                status: 200,
                headers: vec![],
                body: vec![0xff],
            }
            .into_text(10),
            Err(NetError::Transport)
        );
    }

    #[test]
    fn response_body_is_capped() {
        assert_eq!(
            Response {
                status: 200,
                headers: vec![],
                body: vec![1, 2, 3],
            }
            .into_bytes(2),
            Ok(vec![1, 2])
        );
    }
}
