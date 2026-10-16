use jayjay_network::HttpClient;

use super::super::PrLookup;
use super::api::{Api, codeberg_token};
use super::page_scan::{PageScan, PullFilter};
use crate::repo::hosted_repo::HostedRepo;
use crate::types::ChecksStatus;

pub(crate) fn pr_info(remote: &HostedRepo, bookmark: &str) -> PrLookup {
    let api = Api::new(HttpClient::default(), remote, codeberg_token());
    let pr = match api.find_pull(PullFilter::All, |pr| pr.matches(bookmark)) {
        PageScan::Found(pr) => pr,
        PageScan::Exhausted | PageScan::Capped => return PrLookup::NotFound,
        // A private repo 404s the listing and rate limits 429 it; neither is "no PR".
        PageScan::Failed => return PrLookup::Unknown,
    };
    let checks = pr
        .head_sha()
        .and_then(|sha| api.commit_status(sha).ok())
        .map_or(ChecksStatus::None, |status| status.checks());
    PrLookup::Found(pr.into_pr_info(checks))
}
