use super::pull_request::CodebergPrResponse;

// Forgejo's `/pulls` ignores `head=`, so pages are matched client-side.
pub(crate) const PAGE_SIZE: u32 = 50;
const MAX_PAGES: u32 = 10;

pub(crate) enum PageScan {
    Found(CodebergPrResponse),
    Exhausted,
    Failed,
    // Every page up to the cap was full, so a match may exist beyond it.
    Capped,
}

#[derive(Clone, Copy)]
pub(crate) enum PullFilter {
    All,
    Open,
}

impl PullFilter {
    pub(super) fn query(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Open => "open",
        }
    }
}

pub(super) fn scan_pages<E>(
    mut matches: impl FnMut(&CodebergPrResponse) -> bool,
    mut fetch: impl FnMut(u32) -> Result<Vec<CodebergPrResponse>, E>,
) -> PageScan {
    for page in 1..=MAX_PAGES {
        let Ok(prs) = fetch(page) else {
            return PageScan::Failed;
        };
        let last_page = (prs.len() as u32) < PAGE_SIZE;
        if let Some(pr) = prs.into_iter().find(|pr| matches(pr)) {
            return PageScan::Found(pr);
        }
        if last_page {
            return PageScan::Exhausted;
        }
    }
    PageScan::Capped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(heads: impl IntoIterator<Item = String>) -> Vec<CodebergPrResponse> {
        let prs: Vec<String> = heads
            .into_iter()
            .map(|head| format!(r#"{{"number":1,"state":"open","head":{{"ref":"{head}"}}}}"#))
            .collect();
        serde_json::from_str(&format!("[{}]", prs.join(","))).unwrap()
    }

    fn full_page() -> Vec<CodebergPrResponse> {
        page((0..PAGE_SIZE).map(|i| format!("other{i}")))
    }

    fn scan(pages: impl Fn(u32) -> Result<Vec<CodebergPrResponse>, ()>) -> (PageScan, u32) {
        let mut fetched = 0;
        let result = scan_pages(
            |pr| pr.matches("target"),
            |number| {
                fetched = number;
                pages(number)
            },
        );
        (result, fetched)
    }

    #[test]
    fn scan_stops_at_a_match_a_short_page_a_failure_or_the_cap() {
        let (result, fetched) = scan(|number| {
            Ok(if number == 1 {
                full_page()
            } else {
                page(["target".to_owned()])
            })
        });
        assert!(matches!(result, PageScan::Found(_)) && fetched == 2);

        let (result, fetched) = scan(|_| Ok(page(["other".to_owned()])));
        assert!(matches!(result, PageScan::Exhausted) && fetched == 1);

        let (result, fetched) = scan(|number| {
            if number == 1 {
                Ok(full_page())
            } else {
                Err(())
            }
        });
        assert!(matches!(result, PageScan::Failed) && fetched == 2);

        let (result, fetched) = scan(|_| Ok(full_page()));
        assert!(matches!(result, PageScan::Capped) && fetched == MAX_PAGES);
    }
}
