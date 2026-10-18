use jayjay_network::{Auth, Fetch, Method, NetError, Request, Response};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenSource {
    Environment(&'static str),
    Stored,
    Cli(&'static str),
}

impl TokenSource {
    pub fn label(self) -> String {
        match self {
            Self::Environment(variable) => variable.to_owned(),
            Self::Stored => "JayJay sign-in".to_owned(),
            Self::Cli(binary) => format!("{binary} login"),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct ForgeToken {
    pub(crate) secret: String,
    pub(crate) source: TokenSource,
}

impl std::fmt::Debug for ForgeToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ForgeToken")
            .field("source", &self.source)
            .finish_non_exhaustive()
    }
}

pub(crate) trait TokenProvider {
    fn token(&self) -> Option<ForgeToken>;

    /// A different token to retry with after a 401.
    fn refreshed(&self, rejected: &ForgeToken) -> Option<ForgeToken>;

    fn forget(&self) {}

    fn fetch(
        &self,
        fetch: &impl Fetch,
        request: impl Fn(&Auth) -> Request,
    ) -> Result<Response, NetError> {
        let Some(token) = self.token() else {
            return fetch.fetch(&request(&Auth::default()));
        };
        let bearer = |token: &ForgeToken| Auth::bearer(Some(token.secret.clone()));
        let response = fetch.fetch(&request(&bearer(&token)))?;
        if response.status != 401 {
            return Ok(response);
        }
        if let Some(fresh) = self.refreshed(&token) {
            return fetch.fetch(&request(&bearer(&fresh)));
        }
        // A rejected login 401s even public repositories.
        let anonymous = request(&Auth::default());
        if anonymous.method == Method::Get {
            fetch.fetch(&anonymous)
        } else {
            Ok(response)
        }
    }
}

impl TokenProvider for Option<ForgeToken> {
    fn token(&self) -> Option<ForgeToken> {
        self.clone()
    }

    fn refreshed(&self, _: &ForgeToken) -> Option<ForgeToken> {
        None
    }
}

#[cfg(test)]
mod tests {
    use jayjay_network::{DEFAULT_TEXT_CAP, MockFetch};

    use super::*;

    #[test]
    fn a_rejected_token_falls_back_to_an_anonymous_read_but_never_an_anonymous_write() {
        let fetch = MockFetch::new(|request: &Request| {
            let authorized = request.headers.iter().any(|h| h.name == "Authorization");
            Response::ok(if authorized { 401 } else { 200 }, "{}")
        });
        let tokens = Some(ForgeToken {
            secret: "revoked".into(),
            source: TokenSource::Cli("glab"),
        });
        let url = "https://gitlab.com/api/v4/projects/1";

        let read = tokens.fetch(&fetch, |auth| Request::get(url, DEFAULT_TEXT_CAP, auth));
        assert_eq!(read.unwrap().status, 200);
        assert_eq!(fetch.requests().len(), 2);

        let write = tokens.fetch(&fetch, |auth| {
            Request::json(url, Method::Post, auth, &serde_json::json!({})).unwrap()
        });
        assert_eq!(write.unwrap().status, 401);
        assert_eq!(fetch.requests().len(), 3);
    }
}
