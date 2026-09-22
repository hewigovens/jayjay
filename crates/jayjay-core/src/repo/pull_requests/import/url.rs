use gix_url::Scheme;

use crate::repo::hosted_repo::{self, HostedRepo, RepoHost};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParsedPullRequestUrl {
    pub(super) base: HostedRepo,
    pub(super) number: u32,
}

pub(super) fn parse_pull_request_url(raw: &str) -> Option<ParsedPullRequestUrl> {
    let without_query = raw.trim().split(['?', '#']).next()?;
    let url = gix_url::parse(without_query.as_bytes()).ok()?;
    if !matches!(url.scheme, Scheme::Http | Scheme::Https) {
        return None;
    }
    let host = RepoHost::from_name(url.host()?)?;
    let path = std::str::from_utf8(url.path.as_ref()).ok()?;
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    let (owner, repo, number) = match host {
        RepoHost::GitHub => parse_marked_segments(&segments, "pull")?,
        RepoHost::Codeberg => parse_marked_segments(&segments, "pulls")?,
        RepoHost::GitLab => {
            let dash = segments.iter().position(|s| *s == "-")?;
            let namespace = segments[..dash].join("/");
            let rest = &segments[dash + 1..];
            if rest.first() != Some(&"merge_requests") {
                return None;
            }
            let number = rest.get(1)?.parse().ok()?;
            let (owner, repo) = hosted_repo::parse_namespace_repo(&namespace)?;
            (owner, repo, number)
        }
        RepoHost::Cursor => return None,
    };

    Some(ParsedPullRequestUrl {
        base: HostedRepo { host, owner, repo },
        number,
    })
}

fn parse_marked_segments(segments: &[&str], marker: &str) -> Option<(String, String, u32)> {
    let at = segments.iter().position(|s| *s == marker)?;
    let (owner, repo) = hosted_repo::parse_owner_repo(&segments[..at].join("/"))?;
    let number = segments.get(at + 1)?.parse().ok()?;
    Some((owner, repo, number))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(raw: &str) -> Option<ParsedPullRequestUrl> {
        parse_pull_request_url(raw)
    }

    #[test]
    fn parses_github_pull_urls() {
        for raw in [
            "https://github.com/hewigovens/jayjay/pull/290",
            "https://github.com/hewigovens/jayjay/pull/290/files",
            "http://github.com/hewigovens/jayjay/pull/290",
            "https://github.com/hewigovens/jayjay/pull/290#issuecomment-1",
            "  https://github.com/hewigovens/jayjay/pull/290\n",
        ] {
            let parsed = parse(raw).unwrap_or_else(|| panic!("parse {raw}"));
            assert_eq!(parsed.base.host, RepoHost::GitHub);
            assert_eq!(parsed.base.slug(), "hewigovens/jayjay");
            assert_eq!(parsed.number, 290);
        }
    }

    #[test]
    fn parses_gitlab_merge_request_urls() {
        let parsed = parse("https://gitlab.com/group/sub/project/-/merge_requests/45").unwrap();
        assert_eq!(parsed.base.host, RepoHost::GitLab);
        assert_eq!(parsed.base.owner, "group/sub");
        assert_eq!(parsed.base.repo, "project");
        assert_eq!(parsed.number, 45);

        let parsed = parse("https://gitlab.com/owner/repo/-/merge_requests/7/diffs").unwrap();
        assert_eq!(parsed.base.slug(), "owner/repo");
        assert_eq!(parsed.number, 7);
    }

    #[test]
    fn parses_codeberg_pull_urls() {
        let parsed = parse("https://codeberg.org/hewig/jj-test/pulls/12").unwrap();
        assert_eq!(parsed.base.host, RepoHost::Codeberg);
        assert_eq!(parsed.base.slug(), "hewig/jj-test");
        assert_eq!(parsed.number, 12);
    }

    #[test]
    fn rejects_non_pr_and_hostile_urls() {
        for raw in [
            "https://github.com/hewigovens/jayjay",
            "https://github.com/hewigovens/jayjay/pull/",
            "https://github.com/hewigovens/jayjay/pull/abc",
            "https://github.com/hewigovens/jayjay/issues/290",
            "https://github.com/a/b/c/pull/1",
            "https://github.com.evil.org/hewigovens/jayjay/pull/290",
            "https://gitlab.com/owner/repo/merge_requests/45",
            "https://gitlab.com/-/merge_requests/45",
            "https://codeberg.org/o/r/pull/1",
            "https://origin.cursor.com/acme/checkout/pull/1",
            "git@github.com:hewigovens/jayjay/pull/290.git",
            "not a url",
            "",
        ] {
            assert_eq!(parse(raw), None, "{raw}");
        }
    }
}
