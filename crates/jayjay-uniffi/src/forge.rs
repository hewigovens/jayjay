use std::sync::Arc;

use jayjay_core::{DeviceSignIn, JayError, RepoHost, TokenSource};

use crate::repo::JayJaySyncToken;

#[uniffi::remote(Enum)]
pub enum RepoHost {
    GitHub,
    Codeberg,
    GitLab,
    Cursor,
}

#[derive(uniffi::Record)]
pub struct HostLogin {
    pub source: Option<String>,
    pub signed_in: bool,
    pub can_sign_in: bool,
}

#[uniffi::export]
fn token_hosts() -> Vec<RepoHost> {
    RepoHost::TOKEN_HOSTS.to_vec()
}

#[uniffi::export]
fn repo_host_name(host: RepoHost) -> String {
    host.display_name().to_owned()
}

#[uniffi::export]
fn host_login(host: RepoHost) -> HostLogin {
    let login = host.login();
    HostLogin {
        source: login.source.map(TokenSource::label),
        signed_in: login.signed_in,
        can_sign_in: login.can_sign_in,
    }
}

#[uniffi::export]
fn sign_out_of_host(host: RepoHost) -> Result<(), JayError> {
    host.sign_out()
}

#[derive(uniffi::Object)]
pub struct HostSignIn {
    inner: DeviceSignIn,
}

#[uniffi::export]
impl HostSignIn {
    #[uniffi::constructor]
    fn start(host: RepoHost) -> Result<Arc<Self>, JayError> {
        Ok(Arc::new(Self {
            inner: host.start_sign_in()?,
        }))
    }

    fn user_code(&self) -> String {
        self.inner.user_code.clone()
    }

    fn verification_uri(&self) -> String {
        self.inner.verification_uri.clone()
    }

    /// Blocks until approved in the browser; canceling `sync` returns Canceled.
    fn wait(&self, sync: Arc<JayJaySyncToken>) -> Result<(), JayError> {
        self.inner.wait(&sync.inner)
    }
}
