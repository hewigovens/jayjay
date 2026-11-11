use serde::Serialize;

use crate::{CommitAuthor, github_alias, hex_sha256};

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

/// Both shells name the shared on-disk avatar file by this, so an alias shares its GitHub account's file.
pub fn cache_key(email: &str) -> String {
    let alias = github_alias::noreply_email(email);
    email_hash(alias.as_deref().unwrap_or(email))
}

/// The fallback monogram's RGB color, keyed like the avatar file so an alias matches its GitHub account.
pub fn monogram_color(email: &str) -> u32 {
    const PALETTE: [u32; 8] = [
        0x4a5568, 0x6b46c1, 0x2563eb, 0x059669, 0xd97706, 0xdc2626, 0xdb2777, 0x0891b2,
    ];
    let key = cache_key(email);
    let byte = u8::from_str_radix(&key[..2], 16).unwrap_or(0) as usize;
    PALETTE[byte % PALETTE.len()]
}

impl CommitAuthor {
    pub fn monogram_initial(&self) -> char {
        let local = self.email.split('@').next().unwrap_or_default();
        let username = local.rsplit('+').next().unwrap_or_default();
        [self.name.as_str(), username]
            .into_iter()
            .find_map(|text| text.chars().find(|c| c.is_alphanumeric()))
            .and_then(|c| c.to_uppercase().next())
            .unwrap_or('?')
    }
}

#[cfg(test)]
mod tests {
    use super::{AvatarSource, cache_key, email_hash, monogram_color};
    use crate::CommitAuthor;

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
    fn unusual_emails_follow_the_ascii_noreply_rules() {
        assert_eq!(
            source(" 12345+octocat@users.noreply.github.com\n"),
            Some(AvatarSource::Url(
                "https://avatars.githubusercontent.com/u/12345?size=96".to_owned()
            ))
        );
        assert_eq!(
            source("١٢+octocat@users.noreply.github.com"),
            Some(AvatarSource::Url(
                "https://github.com/octocat.png?size=96".to_owned()
            ))
        );
        assert_eq!(
            source("١٢+dependabot[bot]@users.noreply.github.com"),
            Some(AvatarSource::Url(
                "https://github.com/dependabot[bot].png?size=96".to_owned()
            ))
        );
        assert_eq!(
            source("a@b@users.noreply.github.com"),
            Some(AvatarSource::Url(
                "https://github.com/a@b.png?size=96".to_owned()
            ))
        );
        for gravatar in [
            "1-jöhn@users.noreply.gitlab.com",
            "١-john@users.noreply.gitlab.com",
        ] {
            match source(gravatar) {
                Some(AvatarSource::Url(url)) => {
                    assert!(url.starts_with("https://gravatar.com/avatar/"), "{url}")
                }
                other => panic!("expected gravatar fallback for {gravatar}, got {other:?}"),
            }
        }
    }

    #[test]
    fn monogram_color_follows_the_alias_resolved_email() {
        assert_eq!(
            monogram_color("NoReply@Anthropic.com"),
            monogram_color("81847+claude@users.noreply.github.com")
        );
    }

    #[test]
    fn monogram_initial_prefers_the_name_then_the_email_username() {
        let initial =
            |name: &str, email: &str| CommitAuthor::new(name, email, 0).monogram_initial();
        assert_eq!(
            initial("ada lovelace", "12345+octocat@users.noreply.github.com"),
            'A'
        );
        assert_eq!(initial("", "12345+octocat@users.noreply.github.com"), 'O');
        assert_eq!(initial("", ""), '?');
    }

    #[test]
    fn cache_key_normalizes_case_whitespace_and_aliases() {
        assert_eq!(
            cache_key(" Dev@Example.com\n"),
            email_hash("dev@example.com")
        );
        assert_eq!(
            cache_key("noreply@anthropic.com"),
            email_hash("81847+claude@users.noreply.github.com")
        );
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
