use jayjay_network::{Fetch, HttpClient};

use super::super::PrLookup;
use super::api::{Api, codeberg_token};
use super::page_scan::{PageScan, PullFilter};
use super::pull_request::CodebergPrResponse;
use crate::repo::hosted_repo::HostedRepo;
use crate::types::ChecksStatus;

pub(crate) fn pr_info(remote: &HostedRepo, bookmark: &str) -> PrLookup {
    lookup(
        &Api::new(HttpClient::default(), remote, codeberg_token()),
        remote,
        bookmark,
    )
}

fn lookup(api: &Api<impl Fetch>, remote: &HostedRepo, bookmark: &str) -> PrLookup {
    let matches = |pr: &CodebergPrResponse| pr.is_head_of(remote, bookmark);
    // An open PR wins over a newer closed one, as on GitHub and GitLab.
    let scan = match api.find_pull(PullFilter::Open, matches) {
        PageScan::Exhausted => api.find_pull(PullFilter::All, matches),
        scan => scan,
    };
    let pr = match scan {
        PageScan::Found(pr) => pr,
        PageScan::Exhausted => return PrLookup::NotFound,
        // A private repo 404s the listing and rate limits 429 it; neither is "no PR", and a capped scan may have stopped short of it.
        PageScan::Failed | PageScan::Capped => return PrLookup::Unknown,
    };
    let checks = pr
        .head_sha()
        .and_then(|sha| api.commit_status(sha).ok())
        .map_or(ChecksStatus::None, |status| status.checks());
    PrLookup::Found(pr.into_pr_info(checks))
}

#[cfg(test)]
mod tests {
    use jayjay_network::{MockFetch, Response};

    use super::*;
    use crate::repo::pull_requests::codeberg::PAGE_SIZE;
    use crate::types::PrState;

    fn pr(number: u32, state: &str, head: &str, repo: &str) -> String {
        format!(
            r#"{{"number":{number},"state":"{state}","head":{{"ref":"{head}","repo":{{"full_name":"{repo}"}}}}}}"#
        )
    }

    fn page(prs: impl IntoIterator<Item = String>) -> String {
        format!("[{}]", prs.into_iter().collect::<Vec<_>>().join(","))
    }

    fn lookup_with(respond: impl Fn(&str) -> String) -> PrLookup {
        let remote = HostedRepo::parse("https://codeberg.org/owner/repo").unwrap();
        let fetch = MockFetch::new(move |request| Response::ok(200, respond(&request.url)));
        lookup(&Api::new(fetch, &remote, None), &remote, "feat")
    }

    #[test]
    fn lookup_skips_fork_pulls_and_prefers_an_open_one() {
        let found = lookup_with(|url| {
            if url.contains("state=open") {
                page([pr(9, "open", "feat", "fork/repo")])
            } else {
                page([
                    pr(9, "open", "feat", "fork/repo"),
                    pr(4, "closed", "feat", "owner/repo"),
                ])
            }
        });
        assert!(
            matches!(found, PrLookup::Found(info) if info.number == 4 && info.state == PrState::Closed)
        );

        let found = lookup_with(|url| {
            if url.contains("state=open") {
                page([pr(5, "open", "feat", "owner/repo")])
            } else {
                page([pr(6, "closed", "feat", "owner/repo")])
            }
        });
        assert!(matches!(found, PrLookup::Found(info) if info.number == 5));

        let none = lookup_with(|_| page([pr(9, "open", "feat", "fork/repo")]));
        assert!(matches!(none, PrLookup::NotFound));
    }

    #[test]
    fn a_capped_scan_is_inconclusive() {
        let capped =
            lookup_with(|_| page((0..PAGE_SIZE).map(|i| pr(i, "open", "other", "owner/repo"))));
        assert!(matches!(capped, PrLookup::Unknown));
    }
}
