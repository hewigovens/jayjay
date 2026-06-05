mod checks;
mod codeberg;
mod github;

use super::Repo;
use super::hosted_repo::{HostedRepo, RepoHost};
use crate::types::PrInfo;

const PREFERRED_PULL_REQUEST_BASES: &[&str] = &["main", "master", "trunk"];

impl Repo {
    /// Query host-specific PR metadata for a bookmark.
    pub fn pull_request_info(&self, bookmark: &str) -> Option<PrInfo> {
        if bookmark.is_empty() {
            return None;
        }
        let remote = self.git_remote_url().ok()?;
        let remote = HostedRepo::parse(&remote)?;
        self.pull_request_info_for_remote(&remote, bookmark)
    }

    /// Existing PR URL for `bookmark` if one exists, else a supported code-host compose URL.
    pub fn pull_request_open_url(&self, bookmark: &str) -> Option<String> {
        if bookmark.is_empty() {
            return None;
        }
        let remote = self.git_remote_url().ok()?;
        let remote = HostedRepo::parse(&remote)?;
        if let Some(pr) = self.pull_request_info_for_remote(&remote, bookmark) {
            return Some(pr.url);
        }
        let base = if remote.host == RepoHost::Codeberg {
            self.default_pull_request_base()
        } else {
            String::new()
        };
        Some(remote.pull_request_open_url(bookmark, &base))
    }

    pub fn pr_host_name(&self) -> Option<String> {
        let remote = self.git_remote_url().ok()?;
        let remote = HostedRepo::parse(&remote)?;
        Some(remote.host.display_name().to_owned())
    }

    fn default_pull_request_base(&self) -> String {
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

    fn pull_request_info_for_remote(&self, remote: &HostedRepo, bookmark: &str) -> Option<PrInfo> {
        match remote.host {
            RepoHost::GitHub => github::pr_info(self, bookmark),
            RepoHost::Codeberg => codeberg::pr_info(remote, bookmark),
        }
    }
}
