use gix_url::Scheme;

use crate::repo::Repo;
use crate::types::*;

impl Repo {
    /// Get the remote URL for the git repo (origin).
    pub(crate) fn git_remote_url(&self) -> CoreResult<String> {
        let output = self.command_output(
            "git",
            &["remote", "get-url", "origin"],
            "git remote get-url",
        )?;
        self.ensure_success(&output, "git remote get-url")?;
        let url = Self::stdout_text(&output);
        if url.is_empty() {
            return Err(CoreError::Internal {
                message: "No remote 'origin' configured".to_owned(),
            });
        }
        Ok(url)
    }

    /// The origin remote as an https web URL for "open in browser"; `None` if absent or unparseable.
    pub fn remote_web_url(&self) -> Option<String> {
        git_remote_to_web_url(&self.git_remote_url().ok()?)
    }
}

/// Normalize a git remote — scp (`git@host:owner/repo`), `ssh://`, `git://`, or
/// `http(s)://` — to its https web URL. None if it can't be parsed.
fn git_remote_to_web_url(raw: &str) -> Option<String> {
    let url = gix_url::parse(raw.trim().as_bytes().into()).ok()?;
    if !matches!(
        &url.scheme,
        Scheme::Http | Scheme::Https | Scheme::Ssh | Scheme::Git
    ) {
        return None;
    }
    let host = url.host()?;
    let path = std::str::from_utf8(url.path.as_ref())
        .ok()?
        .trim_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    if path.is_empty() {
        return None;
    }
    Some(format!("https://{host}/{path}"))
}

#[cfg(test)]
mod tests {
    use super::git_remote_to_web_url;

    #[test]
    fn git_remote_to_web_url_normalizes_every_form() {
        let cases = [
            (
                "ssh://git@codeberg.org/hewig/jj-test.git",
                "https://codeberg.org/hewig/jj-test",
            ),
            (
                "git@github.com:owner/repo.git",
                "https://github.com/owner/repo",
            ),
            (
                "git://example.com/owner/repo",
                "https://example.com/owner/repo",
            ),
            ("https://codeberg.org/o/r.git", "https://codeberg.org/o/r"),
            ("ssh://git@host:2222/o/r.git", "https://host/o/r"),
        ];
        for (raw, expected) in cases {
            assert_eq!(
                git_remote_to_web_url(raw).as_deref(),
                Some(expected),
                "{raw}"
            );
        }
        assert_eq!(git_remote_to_web_url("not-a-remote"), None);
        assert_eq!(git_remote_to_web_url(""), None);
    }
}
