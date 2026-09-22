use gix::remote::Direction;
use gix_url::Scheme;
use jj_lib::git::{self, get_git_repo};
use jj_lib::ref_name::RemoteName;
use jj_lib::repo::Repo as _;

use crate::repo::Repo;
use crate::repo::hosted_repo::HostedRepo;
use crate::repo::support::unique_name;
use crate::types::*;

pub(crate) struct GitRemote {
    pub(crate) name: String,
    pub(crate) fetch_url: String,
}

impl GitRemote {
    /// Compared scheme-normalized, so an scp remote matches the same repository's https URL.
    pub(crate) fn points_at(&self, url: &str) -> bool {
        let web_url = |raw: &str| git_remote_to_web_url(raw).unwrap_or_else(|| raw.to_owned());
        web_url(&self.fetch_url).eq_ignore_ascii_case(&web_url(url))
    }
}

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

    pub fn remote_web_url(&self) -> Option<String> {
        git_remote_to_web_url(&self.git_remote_url().ok()?)
    }

    pub(crate) fn git_remotes(&self) -> CoreResult<Vec<GitRemote>> {
        let git_repo =
            get_git_repo(self.get_repo().store()).map_err(|error| CoreError::Internal {
                message: format!("read git remotes: {error}"),
            })?;
        Ok(git_repo
            .remote_names()
            .into_iter()
            .filter_map(|name| {
                let remote = git_repo.find_remote(&*name).ok()?;
                // A push-only remote still owns its name, so it stays in the list with no fetch URL to match.
                let fetch_url = remote
                    .url(Direction::Fetch)
                    .map(|url| url.to_bstring().to_string())
                    .unwrap_or_default();
                Some(GitRemote {
                    name: name.to_string(),
                    fetch_url,
                })
            })
            .collect())
    }

    pub(crate) fn git_remote_add(&self, name: &str, url: &str) -> CoreResult<()> {
        if !is_valid_remote_url(url) {
            return Err(CoreError::Internal {
                message: format!("invalid remote URL: {url}"),
            });
        }
        self.with_repo_transaction(&format!("add git remote {name}"), false, |_, mut_repo| {
            git::add_remote(mut_repo, RemoteName::new(name), url, None)
                .map_err(|error| CoreError::internal(format!("add remote {name}: {error}")))
        })
    }
}

/// `hint` is host-controlled, so anything outside a plain remote name folds to `-`.
pub(crate) fn free_remote_name(remotes: &[GitRemote], hint: &str) -> String {
    let sanitized: String = hint
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '-'
            }
        })
        .collect();
    let base = match sanitized.trim_start_matches(['-', '.']) {
        "" => "fork",
        trimmed => trimmed,
    };
    unique_name(base, |candidate| {
        remotes.iter().any(|remote| remote.name == candidate)
    })
}

pub(crate) fn remote_url_uses_ssh(url: &str) -> bool {
    gix_url::parse(url.trim().as_bytes()).is_ok_and(|url| url.scheme == Scheme::Ssh)
}

fn is_valid_remote_url(url: &str) -> bool {
    if url.starts_with('-') {
        return false;
    }
    let Ok(parsed) = gix_url::parse(url.as_bytes()) else {
        return false;
    };
    // Ext and helper transports run local commands at fetch time.
    matches!(
        parsed.scheme,
        Scheme::File | Scheme::Git | Scheme::Http | Scheme::Https | Scheme::Ssh
    )
}

pub(crate) fn git_remote_to_web_url(raw: &str) -> Option<String> {
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
    use super::{
        GitRemote, free_remote_name, git_remote_to_web_url, is_valid_remote_url,
        remote_url_uses_ssh,
    };

    #[test]
    fn only_ssh_origins_prefer_ssh_clone_urls() {
        assert!(remote_url_uses_ssh("git@github.com:alice/repo.git"));
        assert!(remote_url_uses_ssh("ssh://git@github.com/alice/repo.git"));
        assert!(!remote_url_uses_ssh("git://github.com/alice/repo.git"));
        assert!(!remote_url_uses_ssh("https://github.com/alice/repo.git"));
    }

    #[test]
    fn free_remote_name_sanitizes_the_hint_and_skips_taken_names() {
        let taken = |name: &str| GitRemote {
            name: name.into(),
            fetch_url: String::new(),
        };
        assert_eq!(free_remote_name(&[], "-weird name"), "weird-name");
        assert_eq!(free_remote_name(&[], "---"), "fork");
        assert_eq!(
            free_remote_name(&[taken("alice"), taken("alice-2")], "alice"),
            "alice-3"
        );
    }

    #[test]
    fn remote_url_validation_rejects_command_transports_and_options() {
        assert!(is_valid_remote_url("https://github.com/alice/repo.git"));
        assert!(is_valid_remote_url("git@github.com:alice/repo.git"));
        assert!(is_valid_remote_url("ssh://git@gitlab.com/alice/repo.git"));
        assert!(is_valid_remote_url("/tmp/fork.git"));
        assert!(!is_valid_remote_url("--upload-pack=evil"));
        assert!(!is_valid_remote_url("ext::sh -c touch /tmp/pwned"));
        assert!(!is_valid_remote_url(""));
    }

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
