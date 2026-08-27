use std::io::Read;
use std::sync::LazyLock;
use std::time::Duration;

use crate::{Auth, DEFAULT_TEXT_CAP, Header, Method, NetError, Request, Response};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

static AGENT: LazyLock<ureq::Agent> = LazyLock::new(|| {
    ureq::Agent::config_builder()
        .timeout_global(Some(REQUEST_TIMEOUT))
        .http_status_as_error(false)
        .build()
        .into()
});

/// Reusable blocking HTTP client for desktop callers. Run its methods on a background executor.
#[derive(Clone)]
pub struct HttpClient {
    agent: ureq::Agent,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self {
            agent: AGENT.clone(),
        }
    }
}

impl HttpClient {
    pub fn fetch(&self, request: &Request) -> Result<Response, NetError> {
        let method = match request.method {
            Method::Get => ureq::http::Method::GET,
            Method::Post => ureq::http::Method::POST,
            Method::Put => ureq::http::Method::PUT,
            Method::Patch => ureq::http::Method::PATCH,
            Method::Delete => ureq::http::Method::DELETE,
            Method::Head => ureq::http::Method::HEAD,
            Method::Options => ureq::http::Method::OPTIONS,
        };
        let mut outgoing = ureq::http::Request::builder()
            .method(method)
            .uri(&request.url);
        for header in &request.headers {
            outgoing = outgoing.header(&header.name, &header.value);
        }
        let mut response = match &request.body {
            Some(body) => self.agent.run(
                outgoing
                    .body(body.as_slice())
                    .map_err(|_| NetError::Transport)?,
            ),
            None => self
                .agent
                .run(outgoing.body(()).map_err(|_| NetError::Transport)?),
        }
        .map_err(|_| NetError::Transport)?;
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .filter_map(|(name, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|value| Header::new(name.as_str(), value))
            })
            .collect();
        let mut body = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(u64::from(request.max_response_bytes))
            .read_to_end(&mut body)
            .map_err(|_| NetError::Transport)?;
        Ok(Response {
            status,
            headers,
            body,
        })
    }

    pub fn get_text(&self, url: &str) -> Result<String, NetError> {
        self.get_text_with_auth(url, &Auth::default())
    }

    pub fn get_text_with_auth(&self, url: &str, auth: &Auth) -> Result<String, NetError> {
        let request = Request::get(url, DEFAULT_TEXT_CAP, auth);
        self.fetch(&request)?.into_text(request.max_response_bytes)
    }

    pub fn get_bytes(&self, url: &str, max_bytes: u32, auth: &Auth) -> Result<Vec<u8>, NetError> {
        let request = Request::get(url, max_bytes, auth);
        self.fetch(&request)?.into_bytes(request.max_response_bytes)
    }
}
