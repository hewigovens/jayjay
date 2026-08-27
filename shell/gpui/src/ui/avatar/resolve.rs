//! Author email → avatar image URL.

use super::cache::email_md5;

const PIXEL_SIZE: u32 = 96; // 2x for ~24pt slot

/// How an author's avatar resolves: a direct URL, or an id/username needing an API lookup.
#[derive(Debug, PartialEq)]
pub(super) enum AvatarSource {
    Url(String),
    /// A GitHub bot's numeric user id — its avatar lives at `in/<app-id>`, which
    /// only the API maps from the id (`u/<id>` is just an identicon for bots).
    GitHubBot(String),
    /// A gitlab.com username — its avatar is resolved via `users?username=`.
    GitLabUser(String),
}

pub(super) fn avatar_source(email: &str) -> Option<AvatarSource> {
    let trimmed = email.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(local) = trimmed.strip_suffix("@users.noreply.github.com") {
        if let Some((id, user)) = local.split_once('+')
            && !id.is_empty()
            && id.bytes().all(|b| b.is_ascii_digit())
        {
            if user.ends_with("[bot]") {
                return Some(AvatarSource::GitHubBot(id.to_owned()));
            }
            return Some(AvatarSource::Url(format!(
                "https://avatars.githubusercontent.com/u/{id}?size={PIXEL_SIZE}"
            )));
        }
        let user = local.rsplit('+').next().unwrap_or(local);
        if !user.is_empty() {
            return Some(AvatarSource::Url(format!(
                "https://github.com/{user}.png?size={PIXEL_SIZE}"
            )));
        }
    }
    if let Some(local) = trimmed.strip_suffix("@users.noreply.gitlab.com") {
        // `<numeric-id>-<username>` (privacy on) or legacy `<username>`.
        let username = match local.split_once('-') {
            Some((id, rest)) if !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()) => rest,
            _ => local,
        };
        // Validate the charset so a repo-controlled email can't inject query params.
        if is_gitlab_username(username) {
            return Some(AvatarSource::GitLabUser(username.to_owned()));
        }
    }
    Some(AvatarSource::Url(format!(
        "https://gravatar.com/avatar/{}?s={PIXEL_SIZE}&d=retro",
        email_md5(email)
    )))
}

fn is_gitlab_username(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

/// Resolve a GitHub bot's real avatar via the API (`user/<id>` → `avatar_url`).
pub(super) fn bot_avatar_url(user_id: &str) -> Option<String> {
    let api = format!("https://api.github.com/user/{user_id}");
    let bytes = jayjay_network::HttpClient::default()
        .get_bytes(&api, 64 * 1024, &jayjay_network::Auth::default())
        .ok()?;
    let url = json_string_field(std::str::from_utf8(&bytes).ok()?, "avatar_url")?;
    let sep = if url.contains('?') { '&' } else { '?' };
    Some(format!("{url}{sep}size={PIXEL_SIZE}"))
}

/// Resolve a gitlab.com user's avatar via `users?username=` (`avatar_url` = uploaded avatar or Gravatar).
pub(super) fn gitlab_avatar_url(username: &str) -> Option<String> {
    let api = format!("https://gitlab.com/api/v4/users?username={username}");
    let bytes = jayjay_network::HttpClient::default()
        .get_bytes(&api, 64 * 1024, &jayjay_network::Auth::default())
        .ok()?;
    let url = json_string_field(std::str::from_utf8(&bytes).ok()?, "avatar_url")?;
    let sep = if url.contains('?') { '&' } else { '?' };
    Some(format!("{url}{sep}width={PIXEL_SIZE}"))
}

/// Pull a top-level JSON string field (tolerates the whitespace GitHub pretty-prints).
fn json_string_field(json: &str, key: &str) -> Option<String> {
    let key_pat = format!("\"{key}\"");
    let after_key = &json[json.find(&key_pat)? + key_pat.len()..];
    let value = after_key.trim_start().strip_prefix(':')?.trim_start();
    let body = value.strip_prefix('"')?;
    let end = body.find('"')?;
    Some(body[..end].to_owned())
}

#[cfg(test)]
mod tests {
    use super::{AvatarSource, avatar_source, json_string_field};

    #[test]
    fn bot_noreply_resolves_via_api() {
        // A bot's `u/<id>` is only an identicon, so route it through the API lookup.
        assert_eq!(
            avatar_source("49699333+dependabot[bot]@users.noreply.github.com"),
            Some(AvatarSource::GitHubBot("49699333".to_owned()))
        );
    }

    #[test]
    fn user_noreply_resolves_by_user_id() {
        assert_eq!(
            avatar_source("12345+octocat@users.noreply.github.com"),
            Some(AvatarSource::Url(
                "https://avatars.githubusercontent.com/u/12345?size=96".to_owned()
            ))
        );
    }

    #[test]
    fn gitlab_noreply_resolves_via_username() {
        assert_eq!(
            avatar_source("1786152-gitlab-bot@users.noreply.gitlab.com"),
            Some(AvatarSource::GitLabUser("gitlab-bot".to_owned()))
        );
        assert_eq!(
            avatar_source("octocat@users.noreply.gitlab.com"),
            Some(AvatarSource::GitLabUser("octocat".to_owned()))
        );
    }

    #[test]
    fn gitlab_noreply_rejects_injection_username() {
        // A repo-controlled email can't smuggle query params into the API call.
        match avatar_source("1-a&b@users.noreply.gitlab.com") {
            Some(AvatarSource::Url(url)) => {
                assert!(url.starts_with("https://gravatar.com/avatar/"), "{url}");
            }
            other => panic!("expected gravatar fallback, got {other:?}"),
        }
    }

    #[test]
    fn old_noreply_without_id_uses_profile_png() {
        assert_eq!(
            avatar_source("octocat@users.noreply.github.com"),
            Some(AvatarSource::Url(
                "https://github.com/octocat.png?size=96".to_owned()
            ))
        );
    }

    #[test]
    fn other_email_falls_back_to_gravatar() {
        match avatar_source("dev@example.com") {
            Some(AvatarSource::Url(url)) => {
                assert!(url.starts_with("https://gravatar.com/avatar/"), "{url}");
            }
            other => panic!("expected gravatar url, got {other:?}"),
        }
    }

    #[test]
    fn json_string_field_extracts_avatar_url() {
        // GitHub pretty-prints its API responses (note the space after the colon).
        let json = "{\n  \"login\": \"dependabot[bot]\",\n  \"avatar_url\": \"https://avatars.githubusercontent.com/in/29110?v=4\",\n  \"type\": \"Bot\"\n}";
        assert_eq!(
            json_string_field(json, "avatar_url").as_deref(),
            Some("https://avatars.githubusercontent.com/in/29110?v=4")
        );
        assert_eq!(
            json_string_field(r#"{"avatar_url":"https://x/y"}"#, "avatar_url").as_deref(),
            Some("https://x/y")
        );
    }
}
