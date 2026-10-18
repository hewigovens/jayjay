use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jayjay_network::{Fetch, HttpClient, Request};
use serde::{Deserialize, Serialize};

use super::oauth_app::{OAuthApp, TokenReply};
use super::repo_host::TokenSlot;
use super::sign_in_store::SignInStore;
use crate::repo::SyncToken;
use crate::repo::hosted_repo::RepoHost;
use crate::types::{JayError, JayResult};

const KEYCHAIN_SERVICE: &str = "JayJay forge sign-in";
/// Refreshing this early keeps a token from expiring between the check and the request.
const EXPIRY_MARGIN: Duration = Duration::from_secs(60);

static KEYCHAIN_ENABLED: AtomicBool = AtomicBool::new(true);

/// Tests turn this off so they never touch a real credential store.
pub fn use_system_keychain(enabled: bool) {
    KEYCHAIN_ENABLED.store(enabled, Ordering::SeqCst);
}

/// On Linux the Secret Service may be missing.
pub(super) fn keychain_available() -> bool {
    KEYCHAIN_ENABLED.load(Ordering::SeqCst) && SignInStore::system().is_ok()
}

enum Refresh {
    Fresh(StoredLogin),
    Revoked,
    Failed,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(crate) struct StoredLogin {
    pub(crate) access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    /// Seconds since the Unix epoch.
    #[serde(default)]
    expires_at: Option<u64>,
}

impl std::fmt::Debug for StoredLogin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoredLogin")
            .field("expires_at", &self.expires_at)
            .finish_non_exhaustive()
    }
}

impl StoredLogin {
    pub(super) fn from_reply(reply: &TokenReply, now: SystemTime) -> Option<Self> {
        Some(Self {
            access_token: reply.access_token.clone()?,
            refresh_token: reply.refresh_token.clone(),
            expires_at: reply.expires_in.map(|seconds| unix_seconds(now) + seconds),
        })
    }

    fn is_expiring(&self, now: SystemTime) -> bool {
        self.expires_at
            .is_some_and(|at| unix_seconds(now + EXPIRY_MARGIN) >= at)
    }

    fn refreshed(&self, app: &OAuthApp, fetch: &impl Fetch, now: SystemTime) -> Refresh {
        let Some(refresh_token) = self.refresh_token.as_deref() else {
            return Refresh::Failed;
        };
        let request = Request::form(
            app.token_url,
            &[
                ("client_id", app.client_id),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ],
        );
        let Some(reply) = fetch
            .fetch(&request)
            .ok()
            .and_then(|response| response.json::<TokenReply>().ok())
        else {
            return Refresh::Failed;
        };
        match Self::from_reply(&reply, now) {
            Some(fresh) => Refresh::Fresh(fresh),
            None if reply.error.as_deref() == Some("invalid_grant") => Refresh::Revoked,
            None => Refresh::Failed,
        }
    }
}

impl RepoHost {
    /// Refreshes first when about to expire; callers hold the token lock.
    pub(super) fn stored_login(self, force_refresh: bool) -> Option<StoredLogin> {
        let entry = self.keychain_entry().ok()?;
        let login: StoredLogin = serde_json::from_str(&entry.get_password().ok()?).ok()?;
        let now = SystemTime::now();
        if !force_refresh && !login.is_expiring(now) {
            return Some(login);
        }
        let Some(app) = self.oauth_app() else {
            return Some(login);
        };
        match login.refreshed(&app, &HttpClient::default(), now) {
            Refresh::Fresh(fresh) => {
                // An unsaved refresh still works this session; the old refresh token may already be spent.
                if let Ok(json) = serde_json::to_string(&fresh) {
                    let _ = entry.set_password(&json);
                }
                Some(fresh)
            }
            Refresh::Revoked => {
                let _ = entry.delete_credential();
                None
            }
            // Keeping the login lets a later request retry after a transient refresh failure.
            Refresh::Failed => Some(login),
        }
    }

    pub(super) fn store_sign_in(self, login: &StoredLogin, sync: &SyncToken) -> JayResult<()> {
        let json = serde_json::to_string(login).map_err(JayError::internal)?;
        self.with_slot(|slot| {
            let entry = self.keychain_entry()?;
            sync.check()?;
            entry
                .set_password(&json)
                .map_err(|error| keychain_error(self, &error))?;
            *slot = TokenSlot::EMPTY;
            Ok(())
        })
    }

    pub(super) fn has_stored_login(self) -> bool {
        self.with_slot(|_| {
            self.keychain_entry()
                .is_ok_and(|entry| entry.get_credential().is_ok())
        })
    }

    /// A gh/glab login or environment token still applies afterwards.
    pub fn sign_out(self) -> JayResult<()> {
        self.with_slot(|slot| {
            match self.keychain_entry()?.delete_credential() {
                Ok(()) | Err(keyring_core::Error::NoEntry) => {}
                Err(error) => return Err(keychain_error(self, &error)),
            }
            *slot = TokenSlot::EMPTY;
            Ok(())
        })
    }

    fn keychain_entry(self) -> JayResult<keyring_core::Entry> {
        if !KEYCHAIN_ENABLED.load(Ordering::SeqCst) {
            return Err(JayError::internal("The system keychain is turned off."));
        }
        SignInStore::system()
            .as_ref()
            .map_err(|error| keychain_error(self, error))?
            .entry(KEYCHAIN_SERVICE, self.name())
            .map_err(|error| keychain_error(self, &error))
    }
}

fn keychain_error(host: RepoHost, error: &keyring_core::Error) -> JayError {
    JayError::internal(format!(
        "Couldn't use the system keychain for {}: {error}",
        host.display_name()
    ))
}

fn unix_seconds(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[cfg(test)]
mod tests {
    use jayjay_network::{MockFetch, Response};

    use super::*;

    fn reply(json: &str) -> TokenReply {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn an_expiring_login_refreshes_with_its_refresh_token() {
        let now = UNIX_EPOCH + Duration::from_secs(1_000);
        let login = StoredLogin::from_reply(
            &reply(r#"{"access_token":"a1","refresh_token":"r1","expires_in":30}"#),
            now,
        )
        .unwrap();
        assert!(login.is_expiring(now));

        let fetch = MockFetch::new(|_| {
            Response::ok(
                200,
                r#"{"access_token":"a2","refresh_token":"r2","expires_in":7200}"#,
            )
        });
        let app = RepoHost::GitLab.oauth_app().unwrap();
        let Refresh::Fresh(fresh) = login.refreshed(&app, &fetch, now) else {
            panic!("expected a refreshed login");
        };
        assert_eq!(fresh.access_token, "a2");
        assert!(!fresh.is_expiring(now));
        assert!(fetch.requests()[0].body_text().contains("refresh_token=r1"));

        let revoked = MockFetch::new(|_| Response::ok(400, r#"{"error":"invalid_grant"}"#));
        assert!(matches!(
            login.refreshed(&app, &revoked, now),
            Refresh::Revoked
        ));
        let offline = MockFetch::new(|_| Err(jayjay_network::NetError::Transport));
        assert!(matches!(
            login.refreshed(&app, &offline, now),
            Refresh::Failed
        ));
    }

    #[cfg(feature = "mock")]
    #[test]
    fn cancellation_after_the_last_poll_prevents_a_queued_credential_write() {
        let host = RepoHost::Codeberg;
        let login =
            StoredLogin::from_reply(&reply(r#"{"access_token":"approved"}"#), UNIX_EPOCH).unwrap();
        let sync = SyncToken::standalone();
        let waiting = sync.clone();
        let (checked, ready) = std::sync::mpsc::channel();
        let worker = host.with_slot(|_| {
            let worker = std::thread::spawn(move || {
                waiting.check()?;
                checked.send(()).unwrap();
                host.store_sign_in(&login, &waiting)
            });
            ready.recv_timeout(Duration::from_secs(5)).unwrap();
            sync.cancel();
            worker
        });
        let result = worker.join().unwrap();
        let stored = host.keychain_entry().unwrap().get_password().is_ok();
        host.sign_out().unwrap();
        assert!(matches!(result, Err(JayError::Canceled)));
        assert!(!stored);
    }

    #[test]
    fn a_login_without_expiry_or_refresh_token_is_used_as_is() {
        let now = UNIX_EPOCH;
        let login = StoredLogin::from_reply(
            &reply(r#"{"access_token":"gho","token_type":"bearer"}"#),
            now,
        )
        .unwrap();
        assert!(!login.is_expiring(now + Duration::from_secs(10_000_000)));
        let fetch = MockFetch::new(|_| panic!("nothing to refresh"));
        let app = RepoHost::GitHub.oauth_app().unwrap();
        assert!(matches!(
            login.refreshed(&app, &fetch, now),
            Refresh::Failed
        ));
    }
}
