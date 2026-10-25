//! API lookups for avatars that need one; `jayjay_core::avatar` maps emails to sources.

pub(super) const PIXEL_SIZE: u32 = 96; // 2x for ~24pt slot

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
    use super::json_string_field;

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
