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

    /// Existing PR URL for `bookmark`, else a code-host compose URL. None when the
    /// status is unknown so we never propose creating a PR that may already exist.
    pub fn pull_request_open_url(&self, bookmark: &str) -> Option<String> {
        if bookmark.is_empty() {
            return None;
        }
        let remote = self.git_remote_url().ok()?;
        let remote = HostedRepo::parse(&remote)?;
        let lookup = self.pull_request_info_for_remote(&remote, bookmark);
        open_url_for_lookup(lookup, || {
            let base = if remote.host == RepoHost::Codeberg {
                self.default_pull_request_base()
            } else {
                String::new()
            };
            remote.pull_request_open_url(bookmark, &base)
        })
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

/// PR URL when found, compose URL on confirmed absence, None when unknown.
fn open_url_for_lookup(lookup: PrLookup, compose_url: impl FnOnce() -> String) -> Option<String> {
    match lookup {
        PrLookup::Found(pr) => Some(pr.url),
        PrLookup::NotFound => Some(compose_url()),
        PrLookup::Unknown => None,
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
        assert_eq!(url.as_deref(), Some("https://host/pull/1"));
    }

    #[test]
    fn confirmed_absence_falls_back_to_compose() {
        let url = open_url_for_lookup(PrLookup::NotFound, || "COMPOSE".into());
        assert_eq!(url.as_deref(), Some("COMPOSE"));
    }

    #[test]
    fn unknown_never_opens_compose() {
        // Offline / rate-limited: a PR may already exist, so never build compose.
        let url = open_url_for_lookup(PrLookup::Unknown, || panic!("compose must not be built"));
        assert_eq!(url, None);
    }
}
