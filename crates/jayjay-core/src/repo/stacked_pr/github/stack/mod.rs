mod client;
mod reconcile;

use crate::repo::stacked_pr::stack_link_outcome::StackLinkOutcome;
use crate::repo::{Repo, hosted_repo::HostedRepo};
use crate::types::SubmittedLayer;

pub(super) fn reconcile(
    repo: &Repo,
    remote: &HostedRepo,
    layers: &[SubmittedLayer],
) -> Option<StackLinkOutcome> {
    reconcile::reconcile(repo, remote, layers)
}
