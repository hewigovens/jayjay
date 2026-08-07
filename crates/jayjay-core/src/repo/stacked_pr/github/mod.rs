mod pull_request;
mod stack;

use super::NativeStackOutcome;
use super::forge::ForgeTarget;
use crate::repo::{Repo, hosted_repo::HostedRepo};
use crate::types::{CoreResult, SubmittedLayer};

pub(super) fn preflight(repo: &Repo) -> CoreResult<()> {
    pull_request::preflight(repo)
}

pub(super) fn create_or_update_pr(repo: &Repo, target: &ForgeTarget) -> SubmittedLayer {
    pull_request::create_or_update_pr(repo, target)
}

pub(super) fn reconcile_stack(
    repo: &Repo,
    remote: &HostedRepo,
    layers: &[SubmittedLayer],
) -> Option<NativeStackOutcome> {
    stack::reconcile(repo, remote, layers)
}
