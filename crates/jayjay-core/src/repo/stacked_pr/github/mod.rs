mod pull_request;
mod stack;

use super::forge::ForgeTarget;
use crate::repo::stacked_pr::stack_link_outcome::StackLinkOutcome;
use crate::repo::{Repo, hosted_repo::HostedRepo};
use crate::types::{JayResult, SubmittedLayer};

pub(super) fn preflight(repo: &Repo) -> JayResult<()> {
    pull_request::preflight(repo)
}

pub(super) fn create_or_update_pr(repo: &Repo, target: &ForgeTarget) -> SubmittedLayer {
    pull_request::create_or_update_pr(repo, target)
}

pub(super) fn reconcile_stack(
    repo: &Repo,
    remote: &HostedRepo,
    layers: &[SubmittedLayer],
) -> Option<StackLinkOutcome> {
    stack::reconcile(repo, remote, layers)
}
