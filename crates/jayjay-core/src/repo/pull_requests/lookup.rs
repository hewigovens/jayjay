use crate::repo::Repo;
use crate::repo::hosted_repo::{HostedRepo, RepoHost};
use crate::types::{JayError, JayResult, PrInfo};

use super::{codeberg, cursor, github, gitlab};

const PREFERRED_PULL_REQUEST_BASES: &[&str] = &["main", "master", "trunk"];
const NO_SUPPORTED_REMOTE: &str = "Couldn't determine a pull request URL — no GitHub, GitLab, Codeberg, or Cursor \"origin\" remote found.";

pub(in crate::repo) enum PrLookup {
    Found(PrInfo),
    NotFound,
    Unknown,
}

impl Repo {
    pub fn pull_request_info(&self, bookmark: &str) -> Option<PrInfo> {
        match self.pull_request_lookup(bookmark) {
            PrLookup::Found(pr) => Some(pr),
            _ => None,
        }
    }

    pub fn pull_request_open_url(&self, bookmark: &str) -> JayResult<String> {
        if bookmark.is_empty() {
            return Err(JayError::internal("No bookmark selected"));
        }
        let Some(remote) = self
            .git_remote_url()
            .ok()
            .and_then(|url| HostedRepo::parse(&url))
        else {
            return Err(JayError::internal(NO_SUPPORTED_REMOTE));
        };
        let lookup = self.pull_request_info_for_remote(&remote, bookmark);
        match remote.host {
            RepoHost::Cursor => {
                cursor::open_or_create_url(self, bookmark, lookup, remote.web_url())
                    .map_err(JayError::internal)
            }
            _ => Ok(open_url_for_lookup(lookup, || {
                let base = if remote.host == RepoHost::Codeberg {
                    self.default_pull_request_base()
                } else {
                    String::new()
                };
                remote.pull_request_open_url(bookmark, &base)
            })),
        }
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
            RepoHost::Cursor => cursor::pr_info(self, bookmark),
        }
    }
}

/// A failed lookup still composes: the host's new-PR page surfaces an existing PR instead of duplicating it.
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
        let url = open_url_for_lookup(PrLookup::Unknown, || "COMPOSE".into());
        assert_eq!(url, "COMPOSE");
    }
}
