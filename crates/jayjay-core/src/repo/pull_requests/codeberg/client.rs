use jayjay_network::{Auth, NetError};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

use super::super::PrLookup;
use super::pull_request::CodebergPrResponse;
use super::status::CodebergCombinedStatus;
use crate::repo::hosted_repo::HostedRepo;
use crate::types::ChecksStatus;

const CODEBERG_API_URL: &str = "https://codeberg.org/api/v1";
const URL_COMPONENT_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');
/// Forgejo's `/pulls` ignores `head=`, so we page and match client-side.
const PAGE_SIZE: u32 = 50;
/// Cap pages so a huge repo can't stall the status bar. 10 * 50 = 500 PRs.
const MAX_PAGES: u32 = 10;

/// Result of paging through `/pulls`.
enum PageScan {
    Found(CodebergPrResponse),
    /// Every reachable page was read and none matched.
    Exhausted,
    /// A page failed to load; the search is inconclusive.
    Failed,
}

pub(crate) fn pr_info(remote: &HostedRepo, bookmark: &str) -> PrLookup {
    let auth = codeberg_auth();
    let pr = match find_pr(bookmark, |page| {
        let url = pulls_url(remote, page);
        let body = jayjay_network::get_text_with_auth(&url, &auth)?;
        serde_json::from_str::<Vec<CodebergPrResponse>>(&body).map_err(|_| NetError::Transport)
    }) {
        PageScan::Found(pr) => pr,
        PageScan::Exhausted => return PrLookup::NotFound,
        // A private repo 404s the listing and rate limits 429 it; neither is "no PR".
        PageScan::Failed => return PrLookup::Unknown,
    };
    let checks = pr
        .head_sha()
        .and_then(|sha| commit_status(remote, sha, &auth))
        .unwrap_or(ChecksStatus::None);
    PrLookup::Found(pr.into_pr_info(checks))
}

/// Token from `CODEBERG_TOKEN` or `FORGEJO_TOKEN`; without it private repos 404 and rate limits apply.
fn codeberg_auth() -> Auth {
    let token = std::env::var("CODEBERG_TOKEN")
        .or_else(|_| std::env::var("FORGEJO_TOKEN"))
        .ok();
    Auth::token(token)
}

/// Page through `/pulls` and return the first PR whose head matches `bookmark`.
/// `fetch` returns one page, or an error that stops the scan as inconclusive.
fn find_pr(
    bookmark: &str,
    mut fetch: impl FnMut(u32) -> Result<Vec<CodebergPrResponse>, NetError>,
) -> PageScan {
    for page in 1..=MAX_PAGES {
        let prs = match fetch(page) {
            Ok(prs) => prs,
            Err(_) => return PageScan::Failed,
        };
        let short_page = (prs.len() as u32) < PAGE_SIZE;
        if let Some(pr) = prs.into_iter().find(|pr| pr.matches(bookmark)) {
            return PageScan::Found(pr);
        }
        if short_page {
            return PageScan::Exhausted;
        }
    }
    PageScan::Exhausted
}

fn pulls_url(remote: &HostedRepo, page: u32) -> String {
    format!(
        "{}/repos/{}/{}/pulls?state=all&page={}&limit={}",
        CODEBERG_API_URL,
        encode(&remote.owner),
        encode(&remote.repo),
        page,
        PAGE_SIZE,
    )
}

fn commit_status(remote: &HostedRepo, sha: &str, auth: &Auth) -> Option<ChecksStatus> {
    let url = format!(
        "{}/repos/{}/{}/commits/{}/status",
        CODEBERG_API_URL,
        encode(&remote.owner),
        encode(&remote.repo),
        encode(sha)
    );
    let body = jayjay_network::get_text_with_auth(&url, auth).ok()?;
    let combined: CodebergCombinedStatus = serde_json::from_str(&body).ok()?;
    Some(combined.checks())
}

fn encode(s: &str) -> String {
    utf8_percent_encode(s, URL_COMPONENT_ENCODE_SET).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::hosted_repo::RepoHost;

    /// A full page of `count` PRs whose heads are `prefix0..prefixN`.
    fn page(prefix: &str, count: u32) -> Vec<CodebergPrResponse> {
        page_with_heads((0..count).map(|i| format!("{prefix}{i}")))
    }

    fn page_with_heads(heads: impl IntoIterator<Item = String>) -> Vec<CodebergPrResponse> {
        let prs: Vec<String> = heads
            .into_iter()
            .enumerate()
            .map(|(i, head)| pr_json(i as u32, &head))
            .collect();
        serde_json::from_str(&format!("[{}]", prs.join(","))).unwrap()
    }

    fn pr_json(number: u32, head: &str) -> String {
        format!(
            r#"{{"number":{number},"state":"open","title":"t","html_url":"u",
               "head":{{"ref":"{head}","sha":"sha{number}"}}}}"#
        )
    }

    fn remote() -> HostedRepo {
        HostedRepo {
            host: RepoHost::Codeberg,
            owner: "owner".into(),
            repo: "repo".into(),
        }
    }

    impl PageScan {
        fn found(self) -> Option<CodebergPrResponse> {
            match self {
                PageScan::Found(pr) => Some(pr),
                _ => None,
            }
        }

        fn is_exhausted(&self) -> bool {
            matches!(self, PageScan::Exhausted)
        }

        fn is_failed(&self) -> bool {
            matches!(self, PageScan::Failed)
        }
    }

    #[test]
    fn pulls_url_pages_and_drops_head_param() {
        let url = pulls_url(&remote(), 3);
        assert!(url.ends_with("/repos/owner/repo/pulls?state=all&page=3&limit=50"));
        assert!(!url.contains("head="));
    }

    #[test]
    fn finds_match_on_a_later_page() {
        // First full page misses; the bookmark's PR lives on page 2.
        let mut pages_fetched = Vec::new();
        let scan = find_pr("target", |p| {
            pages_fetched.push(p);
            Ok(match p {
                1 => page("other", PAGE_SIZE),
                _ => {
                    let mut heads: Vec<String> =
                        (0..PAGE_SIZE - 1).map(|i| format!("other{i}")).collect();
                    heads.push("target".into());
                    page_with_heads(heads)
                }
            })
        });
        assert!(scan.found().unwrap().matches("target"));
        assert_eq!(pages_fetched, vec![1, 2]);
    }

    #[test]
    fn stops_at_short_page_without_match() {
        // A page shorter than PAGE_SIZE is the last page; don't fetch more.
        let mut pages_fetched = Vec::new();
        let scan = find_pr("missing", |p| {
            pages_fetched.push(p);
            Ok(page("other", PAGE_SIZE - 1))
        });
        assert!(scan.is_exhausted());
        assert_eq!(pages_fetched, vec![1]);
    }

    #[test]
    fn caps_pages_for_full_pages_without_match() {
        // Every page is full and never matches: bounded by MAX_PAGES, exhausted.
        let mut pages_fetched = Vec::new();
        let scan = find_pr("missing", |p| {
            pages_fetched.push(p);
            Ok(page("other", PAGE_SIZE))
        });
        assert!(scan.is_exhausted());
        assert_eq!(pages_fetched.len() as u32, MAX_PAGES);
    }

    #[test]
    fn fetch_failure_is_inconclusive_not_exhausted() {
        // A transport/parse failure mid-paging stops the search as Failed, so the
        // caller treats it as Unknown rather than a confirmed "no PR".
        let mut pages_fetched = Vec::new();
        let scan = find_pr("target", |p| {
            pages_fetched.push(p);
            if p == 1 {
                Ok(page("other", PAGE_SIZE))
            } else {
                Err(NetError::Transport)
            }
        });
        assert!(scan.is_failed());
        assert_eq!(pages_fetched, vec![1, 2]);
    }
}
