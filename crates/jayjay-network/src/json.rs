use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::{Auth, DEFAULT_TEXT_CAP, Header, Method, Request, Response};

impl Request {
    pub fn json(
        url: impl Into<String>,
        method: Method,
        auth: &Auth,
        body: &impl Serialize,
    ) -> serde_json::Result<Self> {
        let mut headers: Vec<Header> = auth.header().into_iter().collect();
        headers.push(Header::new("Content-Type", "application/json"));
        Ok(Self::new(
            url,
            method,
            headers,
            Some(serde_json::to_vec(body)?),
            DEFAULT_TEXT_CAP,
        ))
    }
}

impl Response {
    pub fn json<T: DeserializeOwned>(&self) -> serde_json::Result<T> {
        serde_json::from_slice(&self.body)
    }
}
