use std::cell::{Ref, RefCell};

use crate::{Fetch, Method, NetError, Request, Response};

/// Answers each request from `respond` and records every request, for testing API clients without a network.
pub struct MockFetch<R> {
    respond: R,
    requests: RefCell<Vec<Request>>,
}

impl<R: Fn(&Request) -> Result<Response, NetError>> MockFetch<R> {
    pub fn new(respond: R) -> Self {
        Self {
            respond,
            requests: RefCell::default(),
        }
    }

    pub fn requests(&self) -> Ref<'_, Vec<Request>> {
        self.requests.borrow()
    }

    pub fn writes(&self) -> Vec<Request> {
        self.requests
            .borrow()
            .iter()
            .filter(|request| request.method != Method::Get)
            .cloned()
            .collect()
    }
}

impl<R: Fn(&Request) -> Result<Response, NetError>> Fetch for MockFetch<R> {
    fn fetch(&self, request: &Request) -> Result<Response, NetError> {
        self.requests.borrow_mut().push(request.clone());
        (self.respond)(request)
    }
}

impl Request {
    pub fn body_text(&self) -> &str {
        self.body
            .as_deref()
            .and_then(|body| std::str::from_utf8(body).ok())
            .unwrap_or_default()
    }
}

impl Response {
    pub fn ok(status: u16, body: impl Into<String>) -> Result<Self, NetError> {
        Ok(Self::new(status, body.into()))
    }
}
