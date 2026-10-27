use serde::Serialize;

use crate::{github_alias, hex_sha256};

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum AvatarSource {
    Url(String),
    GitHubBot(String),
    GitLabUser(String),
}

impl AvatarSource {
    pub fn for_email(email: &str, pixel_size: u32) -> Option<Self> {
        let trimmed = email.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Some(noreply) = github_alias::noreply_email(trimmed) {
            return Self::for_email(&noreply, pixel_size);
        }
        if let Some(local) = trimmed.strip_suffix("@users.noreply.github.com") {
            if let Some((id, user)) = local.split_once('+')
                && !id.is_empty()
                && id.bytes().all(|b| b.is_ascii_digit())
            {
                if user.ends_with("[bot]") {
                    return Some(Self::GitHubBot(id.to_owned()));
                }
                return Some(Self::Url(format!(
                    "https://avatars.githubusercontent.com/u/{id}?size={pixel_size}"
                )));
            }
            let user = local.rsplit('+').next().unwrap_or(local);
            if !user.is_empty() {
                return Some(Self::Url(format!(
                    "https://github.com/{user}.png?size={pixel_size}"
                )));
            }
        }
        if let Some(local) = trimmed.strip_suffix("@users.noreply.gitlab.com") {
            let username = match local.split_once('-') {
                Some((id, rest)) if !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()) => {
                    rest
                }
                _ => local,
            };
            if !username.is_empty()
                && username
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
            {
                return Some(Self::GitLabUser(username.to_owned()));
            }
        }
        Some(Self::Url(format!(
            "https://gravatar.com/avatar/{}?s={pixel_size}&d=retro",
            email_hash(email)
        )))
    }
}

pub fn email_hash(email: &str) -> String {
    hex_sha256(email.trim().to_lowercase().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::AvatarSource;

    fn source(email: &str) -> Option<AvatarSource> {
        AvatarSource::for_email(email, 96)
    }

    #[test]
    fn github_noreply_resolves_by_user_id_and_bots_via_the_api() {
        assert_eq!(
            source("12345+octocat@users.noreply.github.com"),
            Some(AvatarSource::Url(
                "https://avatars.githubusercontent.com/u/12345?size=96".to_owned()
            ))
        );
        assert_eq!(
            source("49699333+dependabot[bot]@users.noreply.github.com"),
            Some(AvatarSource::GitHubBot("49699333".to_owned()))
        );
        assert_eq!(
            source("octocat@users.noreply.github.com"),
            Some(AvatarSource::Url(
                "https://github.com/octocat.png?size=96".to_owned()
            ))
        );
    }

    #[test]
    fn alias_email_resolves_to_its_github_account() {
        assert_eq!(
            source("noreply@anthropic.com"),
            Some(AvatarSource::Url(
                "https://avatars.githubusercontent.com/u/81847?size=96".to_owned()
            ))
        );
        assert_eq!(
            source("198982749+Copilot@users.noreply.github.com"),
            Some(AvatarSource::GitHubBot("198982749".to_owned()))
        );
    }

    #[test]
    fn gitlab_noreply_resolves_via_username_and_rejects_injection() {
        assert_eq!(
            source("1786152-gitlab-bot@users.noreply.gitlab.com"),
            Some(AvatarSource::GitLabUser("gitlab-bot".to_owned()))
        );
        assert_eq!(
            source("octocat@users.noreply.gitlab.com"),
            Some(AvatarSource::GitLabUser("octocat".to_owned()))
        );
        match source("1-a&b@users.noreply.gitlab.com") {
            Some(AvatarSource::Url(url)) => {
                assert!(url.starts_with("https://gravatar.com/avatar/"), "{url}")
            }
            other => panic!("expected gravatar fallback, got {other:?}"),
        }
    }

    #[test]
    fn other_email_falls_back_to_gravatar_and_empty_has_none() {
        match source("dev@example.com") {
            Some(AvatarSource::Url(url)) => {
                assert!(url.starts_with("https://gravatar.com/avatar/"), "{url}")
            }
            other => panic!("expected gravatar url, got {other:?}"),
        }
        assert_eq!(source("  "), None);
    }

    #[test]
    fn serializes_as_tagged_kind_and_value() {
        assert_eq!(
            serde_json::to_string(&source("49699333+dependabot[bot]@users.noreply.github.com"))
                .unwrap(),
            r#"{"kind":"gitHubBot","value":"49699333"}"#
        );
    }
}
