mod checks;
mod codeberg;
mod github;
mod gitlab;

use super::Repo;
use super::hosted_repo::{HostedRepo, RepoHost};
use crate::types::PrInfo;

const PREFERRED_PULL_REQUEST_BASES: &[&str] = &["main", "master", "trunk"];

/// Outcome of a host PR lookup. A failed call must stay distinct from a confirmed
/// "no PR" so it never triggers a compose-URL fallback.
pub(super) enum PrLookup {
    /// The host returned a PR for this bookmark.
    Found(PrInfo),
    /// The host confirmed there is no PR (empty list / "no pull requests found").
    NotFound,
    /// The lookup could not complete (offline, rate limited, auth, 5xx).
    Unknown,
}

impl Repo {
    /// Query host-specific PR metadata for a bookmark.
    pub fn pull_request_info(&self, bookmark: &str) -> Option<PrInfo> {
        match self.pull_request_lookup(bookmark) {
            PrLookup::Found(pr) => Some(pr),
            _ => None,
        }
    }

    /// Existing PR URL for `bookmark`, else the code host's new-PR (compose) URL.
    /// `None` only when there is no supported `origin` remote to build a URL from.
    pub fn pull_request_open_url(&self, bookmark: &str) -> Option<String> {
        if bookmark.is_empty() {
            return None;
        }
        let remote = self.git_remote_url().ok()?;
        let remote = HostedRepo::parse(&remote)?;
        let lookup = self.pull_request_info_for_remote(&remote, bookmark);
        Some(open_url_for_lookup(lookup, || {
            let base = if remote.host == RepoHost::Codeberg {
                self.default_pull_request_base()
            } else {
                String::new()
            };
            remote.pull_request_open_url(bookmark, &base)
        }))
    }

    fn pull_request_lookup(&self, bookmark: &str) -> PrLookup {
        if bookmark.is_empty() {
            return PrLookup::NotFound;
        }
        let Ok(remote) = self.git_remote_url() else {
            return PrLookup::Unknown;
        };
        let Some(remote) = HostedRepo::parse(&remote) else {
            return PrLookup::NotFound;
        };
        self.pull_request_info_for_remote(&remote, bookmark)
    }

    pub fn pr_host_name(&self) -> Option<String> {
        let remote = self.git_remote_url().ok()?;
        let remote = HostedRepo::parse(&remote)?;
        Some(remote.host.display_name().to_owned())
    }

    pub(crate) fn default_pull_request_base(&self) -> String {
        let Ok(bookmarks) = self.list_bookmarks() else {
            return "main".to_owned();
        };
        PREFERRED_PULL_REQUEST_BASES
            .iter()
            .find(|base| {
                bookmarks
                    .iter()
                    .any(|bookmark| bookmark.name == **base && !bookmark.is_deleted)
            })
            .unwrap_or(&"main")
            .to_string()
    }

    fn pull_request_info_for_remote(&self, remote: &HostedRepo, bookmark: &str) -> PrLookup {
        match remote.host {
            RepoHost::GitHub => github::pr_info(self, bookmark),
            RepoHost::Codeberg => codeberg::pr_info(remote, bookmark),
            RepoHost::GitLab => gitlab::pr_info(remote, bookmark),
        }
    }
}

/// Existing PR URL when found, otherwise the host's new-PR (compose) URL.
///
/// We compose even when the lookup could not complete (`gh` missing or unauthenticated, offline, rate limited). The new-PR pages on GitHub, GitLab, and Codeberg surface an existing PR for the branch rather than silently creating a duplicate, so a working "Pull Request" action beats a dead one. The PR *badge* (`pull_request_info`) still treats `Unknown` as "no PR" so it never shows status it could not confirm.
fn open_url_for_lookup(lookup: PrLookup, compose_url: impl FnOnce() -> String) -> String {
    match lookup {
        PrLookup::Found(pr) => pr.url,
        PrLookup::NotFound | PrLookup::Unknown => compose_url(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChecksStatus, PrState};

    fn pr() -> PrInfo {
        PrInfo {
            number: 1,
            state: PrState::Open,
            title: "t".into(),
            url: "https://host/pull/1".into(),
            checks: ChecksStatus::None,
        }
    }

    #[test]
    fn found_uses_pr_url_not_compose() {
        let url = open_url_for_lookup(PrLookup::Found(pr()), || "COMPOSE".into());
        assert_eq!(url, "https://host/pull/1");
    }

    #[test]
    fn confirmed_absence_falls_back_to_compose() {
        let url = open_url_for_lookup(PrLookup::NotFound, || "COMPOSE".into());
        assert_eq!(url, "COMPOSE");
    }

    #[test]
    fn unknown_falls_back_to_compose() {
        // gh missing/unauthenticated or offline: still open the host's new-PR page. It surfaces an existing PR instead of duplicating, so the button works rather than dying with a misleading "push first" message.
        let url = open_url_for_lookup(PrLookup::Unknown, || "COMPOSE".into());
        assert_eq!(url, "COMPOSE");
    }
}
