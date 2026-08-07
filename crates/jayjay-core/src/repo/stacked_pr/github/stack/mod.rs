mod client;
mod reconcile;

use crate::repo::stacked_pr::NativeStackOutcome;
use crate::repo::{Repo, hosted_repo::HostedRepo};
use crate::types::SubmittedLayer;

pub(super) fn reconcile(
    repo: &Repo,
    remote: &HostedRepo,
    layers: &[SubmittedLayer],
) -> Option<NativeStackOutcome> {
    reconcile::reconcile(repo, remote, layers)
}
