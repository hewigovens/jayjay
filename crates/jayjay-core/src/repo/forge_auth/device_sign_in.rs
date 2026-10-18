use std::time::{Duration, Instant, SystemTime};

use jayjay_network::{Fetch, HttpClient, Request};
use serde::Deserialize;

use super::oauth_app::{OAuthApp, TokenReply};
use super::stored_login::StoredLogin;
use crate::repo::SyncToken;
use crate::repo::hosted_repo::RepoHost;
use crate::types::{JayError, JayResult};

const DEVICE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";
/// RFC 8628's slow_down step.
const SLOW_DOWN_STEP: Duration = Duration::from_secs(5);
const CANCEL_CHECK: Duration = Duration::from_millis(200);

pub struct DeviceSignIn {
    pub host: RepoHost,
    pub user_code: String,
    pub verification_uri: String,
    app: OAuthApp,
    device_code: String,
    interval: Duration,
    expires_at: Instant,
}

#[derive(Deserialize)]
struct DeviceCode {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    #[serde(default = "default_interval")]
    interval: u64,
}

fn default_interval() -> u64 {
    5
}

#[derive(Debug, PartialEq)]
enum Poll {
    Pending,
    SlowDown,
    Done(StoredLogin),
}

impl RepoHost {
    pub fn start_sign_in(self) -> JayResult<DeviceSignIn> {
        self.start_sign_in_with(&HttpClient::default())
    }

    fn start_sign_in_with(self, fetch: &impl Fetch) -> JayResult<DeviceSignIn> {
        let app = self.oauth_app().ok_or_else(|| {
            JayError::internal(format!(
                "Signing in to {} isn't supported.",
                self.display_name()
            ))
        })?;
        let request = Request::form(
            app.device_code_url,
            &[("client_id", app.client_id), ("scope", app.scope)],
        );
        let code: DeviceCode = fetch
            .fetch(&request)
            .map_err(|error| self.sign_in_error(error))?
            .error_for_status()
            .map_err(|response| self.sign_in_error(format!("HTTP {}", response.status)))?
            .json()
            .map_err(|error| self.sign_in_error(error))?;
        Ok(DeviceSignIn {
            host: self,
            user_code: code.user_code,
            verification_uri: code.verification_uri,
            app,
            device_code: code.device_code,
            interval: Duration::from_secs(code.interval.max(1)),
            expires_at: Instant::now() + Duration::from_secs(code.expires_in),
        })
    }

    fn sign_in_error(self, detail: impl std::fmt::Display) -> JayError {
        JayError::internal(format!(
            "Couldn't start signing in to {}: {detail}",
            self.display_name()
        ))
    }
}

impl DeviceSignIn {
    /// `sync` cancels the wait.
    pub fn wait(&self, sync: &SyncToken) -> JayResult<()> {
        self.wait_with(&HttpClient::default(), sync)
    }

    fn wait_with(&self, fetch: &impl Fetch, sync: &SyncToken) -> JayResult<()> {
        let mut interval = self.interval;
        loop {
            sleep_unless_canceled(interval, sync)?;
            if Instant::now() >= self.expires_at {
                return Err(JayError::internal(
                    "The sign-in code expired. Start signing in again.",
                ));
            }
            match self.poll(fetch)? {
                Poll::Pending => {}
                Poll::SlowDown => interval += SLOW_DOWN_STEP,
                Poll::Done(login) => return self.host.store_sign_in(&login, sync),
            }
        }
    }

    fn poll(&self, fetch: &impl Fetch) -> JayResult<Poll> {
        let request = Request::form(
            self.app.token_url,
            &[
                ("client_id", self.app.client_id),
                ("device_code", &self.device_code),
                ("grant_type", DEVICE_GRANT),
            ],
        );
        // GitHub answers a pending poll with 200 and GitLab with 400; both carry the OAuth error code.
        let Ok(response) = fetch.fetch(&request) else {
            return Ok(Poll::Pending);
        };
        let reply: TokenReply = match response.json() {
            Ok(reply) => reply,
            // Rate limits and outage pages are transient.
            Err(_) if response.status == 429 || response.status >= 500 => return Ok(Poll::Pending),
            Err(_) => {
                return Err(JayError::internal(format!(
                    "HTTP {} while signing in",
                    response.status
                )));
            }
        };
        if let Some(login) = StoredLogin::from_reply(&reply, SystemTime::now()) {
            return Ok(Poll::Done(login));
        }
        match reply.error.as_deref() {
            Some("authorization_pending") => Ok(Poll::Pending),
            Some("slow_down") => Ok(Poll::SlowDown),
            Some("expired_token") => Err(JayError::internal(
                "The sign-in code expired. Start signing in again.",
            )),
            Some("access_denied") => Err(JayError::internal(format!(
                "The sign-in was declined on {}.",
                self.host.display_name()
            ))),
            _ => Err(JayError::internal(format!(
                "{} didn't complete the sign-in: {}",
                self.host.display_name(),
                reply
                    .error_description
                    .or(reply.error)
                    .unwrap_or_else(|| "no token in the answer".to_owned())
            ))),
        }
    }
}

fn sleep_unless_canceled(duration: Duration, sync: &SyncToken) -> JayResult<()> {
    let until = Instant::now() + duration;
    while Instant::now() < until {
        sync.check()?;
        std::thread::sleep(CANCEL_CHECK.min(until - Instant::now()));
    }
    sync.check()
}

#[cfg(test)]
mod tests {
    use jayjay_network::{MockFetch, NetError, Response};

    use super::*;

    fn sign_in(host: RepoHost) -> DeviceSignIn {
        let fetch = MockFetch::new(|_| {
            Response::ok(
                200,
                r#"{"device_code":"dc","user_code":"ABCD-1234","verification_uri":"https://github.com/login/device","expires_in":900,"interval":5}"#,
            )
        });
        let sign_in = host.start_sign_in_with(&fetch).unwrap();
        let body = fetch.requests()[0].body_text().to_owned();
        assert!(
            body.contains("client_id=") && body.contains("scope="),
            "{body}"
        );
        sign_in
    }

    fn poll(host: RepoHost, status: u16, body: &'static str) -> JayResult<Poll> {
        let sign_in = sign_in(host);
        let fetch = MockFetch::new(move |_| Response::ok(status, body));
        sign_in.poll(&fetch)
    }

    #[test]
    fn start_returns_the_code_the_user_enters() {
        let sign_in = sign_in(RepoHost::GitHub);
        assert_eq!(sign_in.user_code, "ABCD-1234");
        assert_eq!(sign_in.verification_uri, "https://github.com/login/device");
        assert!(RepoHost::Codeberg.start_sign_in().is_err());
    }

    #[test]
    fn polls_map_each_oauth_answer() {
        assert_eq!(
            poll(
                RepoHost::GitHub,
                200,
                r#"{"error":"authorization_pending"}"#
            )
            .unwrap(),
            Poll::Pending
        );
        assert_eq!(
            poll(
                RepoHost::GitLab,
                400,
                r#"{"error":"authorization_pending"}"#
            )
            .unwrap(),
            Poll::Pending
        );
        assert_eq!(
            poll(
                RepoHost::GitHub,
                200,
                r#"{"error":"slow_down","interval":10}"#
            )
            .unwrap(),
            Poll::SlowDown
        );
        let Poll::Done(login) = poll(
            RepoHost::GitHub,
            200,
            r#"{"access_token":"gho_x","token_type":"bearer","scope":"repo"}"#,
        )
        .unwrap() else {
            panic!("expected a token");
        };
        assert_eq!(login.access_token, "gho_x");
        let declined = poll(RepoHost::GitHub, 200, r#"{"error":"access_denied"}"#).unwrap_err();
        assert!(declined.to_string().contains("declined"));
        let expired = poll(RepoHost::GitLab, 400, r#"{"error":"expired_token"}"#).unwrap_err();
        assert!(expired.to_string().contains("expired"));
    }

    #[test]
    fn transient_poll_failures_keep_waiting_but_a_bad_answer_stops() {
        let sign_in = sign_in(RepoHost::GitHub);
        let dropped = MockFetch::new(|_| Err(NetError::Transport));
        assert_eq!(sign_in.poll(&dropped).unwrap(), Poll::Pending);
        assert_eq!(
            poll(RepoHost::GitLab, 429, "Retry later").unwrap(),
            Poll::Pending
        );
        assert_eq!(
            poll(RepoHost::GitLab, 502, "<html>").unwrap(),
            Poll::Pending
        );
        assert!(poll(RepoHost::GitLab, 404, "<html>").is_err());
    }

    #[test]
    fn a_canceled_wait_stops_before_polling() {
        let sync = SyncToken::standalone();
        sync.cancel();
        let fetch = MockFetch::new(|_| panic!("a canceled wait must not poll"));
        assert!(matches!(
            sign_in(RepoHost::GitHub).wait_with(&fetch, &sync),
            Err(JayError::Canceled)
        ));
    }
}
