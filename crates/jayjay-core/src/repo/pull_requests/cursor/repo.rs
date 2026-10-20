use serde::Deserialize;

use super::super::ApiError;

pub(super) const GITHUB_MIRROR_BLOCK: &str =
    "Cursor pull requests aren't available for GitHub-mirrored repositories.";

#[derive(Deserialize)]
pub(super) struct OriginRepo {
    #[serde(default, rename = "defaultBranch")]
    default_branch: String,
    #[serde(default)]
    mirror: Option<OriginMirror>,
}

#[derive(Deserialize)]
struct OriginMirror {
    status: String,
}

impl OriginRepo {
    pub(super) fn pull_request_base(&self) -> Result<String, String> {
        if self
            .mirror
            .as_ref()
            .is_some_and(|mirror| mirror.status.eq_ignore_ascii_case("inbound"))
        {
            return Err(GITHUB_MIRROR_BLOCK.to_owned());
        }
        let base = self.default_branch.trim();
        if base.is_empty() {
            Err("Cursor didn't return the repository's default branch".to_owned())
        } else {
            Ok(base.to_owned())
        }
    }
}

/// Origin rejects pull requests on inbound GitHub mirrors (`mirrorStatus="inbound"`).
pub(super) fn create_failure_message(error: &ApiError) -> String {
    let message = error.to_string();
    let lower = message.to_ascii_lowercase();
    if lower.contains("github-mirrored") || lower.contains("mirrorstatus=\"inbound\"") {
        GITHUB_MIRROR_BLOCK.to_owned()
    } else {
        message
    }
}

#[cfg(test)]
mod tests {
    use jayjay_network::Response;

    use super::*;

    fn base(json: &str) -> Result<String, String> {
        serde_json::from_str::<OriginRepo>(json)
            .unwrap()
            .pull_request_base()
    }

    #[test]
    fn inbound_github_mirrors_cannot_host_pull_requests() {
        let inbound = r#"{"defaultBranch":"main","mirror":{"source":"github","status":"inbound"}}"#;
        assert_eq!(base(inbound).unwrap_err(), GITHUB_MIRROR_BLOCK);
        assert_eq!(
            base(r#"{"defaultBranch":"main","mirror":{"status":"INBOUND"}}"#).unwrap_err(),
            GITHUB_MIRROR_BLOCK
        );
        assert_eq!(base(r#"{"defaultBranch":"develop"}"#).unwrap(), "develop");
        assert!(base(r#"{"mirror":null}"#).is_err());
    }

    #[test]
    fn create_failure_rewrites_the_mirror_error() {
        let mirror = Response::new(
            400,
            r#"{"code":9,"message":"Origin pull requests are not available for GitHub-mirrored repos; got mirrorStatus=\"inbound\"."}"#,
        );
        assert_eq!(
            create_failure_message(&ApiError::from_response(&mirror)),
            GITHUB_MIRROR_BLOCK
        );
        let missing = Response::new(400, r#"{"code":3,"message":"ref \"x\" does not exist"}"#);
        assert_eq!(
            create_failure_message(&ApiError::from_response(&missing)),
            "HTTP 400: ref \"x\" does not exist"
        );
    }
}
