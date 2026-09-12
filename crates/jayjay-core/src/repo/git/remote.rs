use gix::remote::Direction;
use gix_url::Scheme;
use jj_lib::git::get_git_repo;
use jj_lib::repo::Repo as _;

use crate::repo::Repo;
use crate::repo::hosted_repo::HostedRepo;
use crate::types::*;

impl Repo {
    /// Read from the git config of the store jj uses; `git remote get-url` spawns a subprocess and resolves against the working directory instead.
    pub(crate) fn git_remote_url(&self) -> CoreResult<String> {
        let missing = || CoreError::Internal {
            message: "No remote 'origin' configured".to_owned(),
        };
        let git_repo =
            get_git_repo(self.get_repo().store()).map_err(|error| CoreError::Internal {
                message: format!("read git remote: {error}"),
            })?;
        let remote = git_repo.find_remote("origin").map_err(|_| missing())?;
        let url = remote.url(Direction::Fetch).ok_or_else(missing)?;
        Ok(url.to_bstring().to_string())
    }

    /// The origin remote as an https web URL for "open in browser"; `None` if absent or unparseable.
    pub fn remote_web_url(&self) -> Option<String> {
        git_remote_to_web_url(&self.git_remote_url().ok()?)
    }
}

/// Normalize a git remote — scp (`git@host:owner/repo`), `ssh://`, `git://`, or
/// `http(s)://` — to its https web URL. None if it can't be parsed.
fn git_remote_to_web_url(raw: &str) -> Option<String> {
    let url = gix_url::parse(raw.trim().as_bytes()).ok()?;
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
    if let Some(remote) = HostedRepo::parse(raw) {
        return Some(remote.web_url());
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
            (
                "https://origin.cursor.com/acme/checkout.git",
                "https://cursor.com/codebase/acme/checkout",
            ),
            (
                "git@origin.cursor.com:acme/checkout.git",
                "https://cursor.com/codebase/acme/checkout",
            ),
            (
                "https://origin.cursor.com/git/acme/checkout.git",
                "https://cursor.com/codebase/acme/checkout",
            ),
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
