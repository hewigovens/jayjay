use super::stored_login::keychain_available;
use super::token::TokenSource;
use crate::repo::hosted_repo::RepoHost;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostLogin {
    pub host: RepoHost,
    pub source: Option<TokenSource>,
    pub signed_in: bool,
    pub can_sign_in: bool,
}

impl RepoHost {
    /// May run a CLI or read the keychain; call it off the UI thread.
    pub fn login(self) -> HostLogin {
        HostLogin {
            host: self,
            source: self.token_source(),
            signed_in: self.has_stored_login(),
            can_sign_in: self.oauth_app().is_some() && keychain_available(),
        }
    }
}
