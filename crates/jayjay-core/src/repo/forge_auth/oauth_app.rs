use serde::Deserialize;

use crate::repo::hosted_repo::RepoHost;

/// Device-flow client IDs are public by design.
pub(crate) struct OAuthApp {
    pub(crate) client_id: &'static str,
    pub(crate) device_code_url: &'static str,
    pub(crate) token_url: &'static str,
    pub(crate) scope: &'static str,
}

#[derive(Deserialize)]
pub(crate) struct TokenReply {
    pub(crate) access_token: Option<String>,
    pub(crate) refresh_token: Option<String>,
    pub(crate) expires_in: Option<u64>,
    pub(crate) error: Option<String>,
    pub(crate) error_description: Option<String>,
}

impl RepoHost {
    pub(crate) fn oauth_app(self) -> Option<OAuthApp> {
        match self {
            Self::GitHub => Some(OAuthApp {
                client_id: "Ov23lidy2GfgkLiwRiBh",
                device_code_url: "https://github.com/login/device/code",
                token_url: "https://github.com/login/oauth/access_token",
                scope: "repo read:org",
            }),
            Self::GitLab => Some(OAuthApp {
                client_id: "6750e2410f1da2375c40bceb1c629fbc6e7792a760ba435bda24c8b3793ce1ad",
                device_code_url: "https://gitlab.com/oauth/authorize_device",
                token_url: "https://gitlab.com/oauth/token",
                scope: "api",
            }),
            Self::Codeberg | Self::Cursor => None,
        }
    }
}
