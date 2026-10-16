use jayjay_network::HttpClient;

use super::codeberg::CodebergStack;
use super::forge::ForgeTarget;
use super::stack_link_outcome::StackLinkOutcome;
use super::{cursor, github, gitlab};
use crate::repo::Repo;
use crate::repo::hosted_repo::{HostedRepo, RepoHost};
use crate::repo::pull_requests::codeberg::codeberg_token;
use crate::types::{JayError, JayResult, SubmittedLayer};

pub(super) struct StackForge {
    remote: HostedRepo,
}

impl StackForge {
    pub(super) fn for_remote(remote: Option<HostedRepo>) -> JayResult<Self> {
        remote.map(|remote| Self { remote }).ok_or_else(|| {
            JayError::internal("Stacked PRs support GitHub, GitLab, Codeberg, and Cursor remotes.")
        })
    }

    pub(super) fn preflight(&self, repo: &Repo) -> JayResult<String> {
        match self.remote.host {
            RepoHost::GitHub => {
                github::preflight(repo)?;
                Ok(repo.default_pull_request_base())
            }
            RepoHost::GitLab => {
                gitlab::preflight(repo)?;
                Ok(repo.default_pull_request_base())
            }
            RepoHost::Cursor => cursor::preflight(repo),
            RepoHost::Codeberg => self.codeberg().preflight(),
        }
    }

    pub(super) fn submit_layer(&self, repo: &Repo, target: &ForgeTarget) -> SubmittedLayer {
        match self.remote.host {
            RepoHost::GitHub => github::create_or_update_pr(repo, target),
            RepoHost::GitLab => gitlab::create_or_update_mr(repo, target),
            RepoHost::Cursor => cursor::create_or_update_pr(repo, target),
            RepoHost::Codeberg => self.codeberg().submit_layer(target),
        }
    }

    pub(super) fn link(
        &self,
        repo: &Repo,
        targets: &[ForgeTarget],
        layers: &[SubmittedLayer],
    ) -> Option<StackLinkOutcome> {
        match self.remote.host {
            RepoHost::GitHub => github::reconcile_stack(repo, &self.remote, layers),
            RepoHost::Codeberg => Some(match self.codeberg().link(targets, layers) {
                Ok(()) => StackLinkOutcome::Linked(String::new()),
                Err(notes) => StackLinkOutcome::Fallback(notes),
            }),
            RepoHost::GitLab | RepoHost::Cursor => None,
        }
    }

    pub(super) fn open_urls(&self, layers: &[SubmittedLayer], linked: bool) -> Vec<String> {
        let mut urls = layers
            .iter()
            .map(|layer| layer.pr_url.as_str())
            .filter(|url| !url.is_empty());
        // GitLab and a linked stack reach every layer from the top one, so one page is enough.
        let top_only = self.remote.host == RepoHost::GitLab
            || (linked && layers.last().is_some_and(|layer| !layer.pr_url.is_empty()));
        if top_only {
            urls.next_back().map(str::to_owned).into_iter().collect()
        } else {
            urls.map(str::to_owned).collect()
        }
    }

    fn codeberg(&self) -> CodebergStack<'_, HttpClient> {
        CodebergStack::new(HttpClient::default(), &self.remote, codeberg_token())
    }
}
