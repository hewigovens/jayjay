use jayjay_network::HttpClient;

use super::{FlockStore, FlockVerification};
use crate::repo::forge_auth::{ForgeToken, TokenSource};
use crate::{DeviceSignIn, JayResult, SyncToken};

pub struct FlockSignIn {
    inner: DeviceSignIn,
    sponsorable: String,
}

impl FlockSignIn {
    pub fn start(sponsorable: String) -> JayResult<Self> {
        Ok(Self {
            inner: DeviceSignIn::start_flock()?,
            sponsorable,
        })
    }

    pub fn user_code(&self) -> &str {
        &self.inner.user_code
    }

    pub fn verification_uri(&self) -> &str {
        &self.inner.verification_uri
    }

    pub fn wait(&self, store: &mut FlockStore, sync: &SyncToken) -> JayResult<FlockVerification> {
        let token = self.inner.wait_for_token(sync)?;
        // A Flock-only login never replaces the repository credential or persists its access token.
        let tokens = Some(ForgeToken {
            secret: token,
            source: TokenSource::Stored,
        });
        store.verify_with(&self.sponsorable, &HttpClient::default(), &tokens, sync)
    }
}
