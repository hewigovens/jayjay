use gix_url::Scheme;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

const GITHUB_HOST: &str = "github.com";
const CODEBERG_HOST: &str = "codeberg.org";
const GITLAB_HOST: &str = "gitlab.com";
const DEFAULT_PULL_REQUEST_BASE: &str = "main";
const PULL_REQUEST_REF_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~')
    .remove(b'/');

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HostedRepo {
    pub(crate) host: RepoHost,
    pub(crate) owner: String,
    pub(crate) repo: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RepoHost {
    GitHub,
    Codeberg,
    GitLab,
}

impl HostedRepo {
    /// Parse a Git remote URL and keep only supported repository hosts.
    pub(crate) fn parse(raw: &str) -> Option<Self> {
        let remote = gix_url::parse(raw.trim().as_bytes().into()).ok()?;
        if !matches!(
            &remote.scheme,
            Scheme::Http | Scheme::Https | Scheme::Ssh | Scheme::Git
        ) {
            return None;
        }

        let host = remote.host()?;
        let host = RepoHost::from_name(host)?;
        let path = std::str::from_utf8(remote.path.as_ref()).ok()?;
        let (owner, repo) = parse_owner_repo(path)?;

        Some(Self { host, owner, repo })
    }

    pub(crate) fn pull_request_open_url(&self, bookmark: &str, base: &str) -> String {
        let encoded_bookmark = encode_pull_request_ref(bookmark);
        match self.host {
            RepoHost::GitHub => {
                format!(
                    "https://{}/{}/pull/new/{}",
                    self.host.name(),
                    self.slug(),
                    encoded_bookmark
                )
            }
            RepoHost::Codeberg => {
                let base = if base.is_empty() {
                    DEFAULT_PULL_REQUEST_BASE
                } else {
                    base
                };
                let encoded_base = encode_pull_request_ref(base);
                format!(
                    "https://{}/{}/compare/{}...{}",
                    self.host.name(),
                    self.slug(),
                    encoded_base,
                    encoded_bookmark
                )
            }
            RepoHost::GitLab => {
                // GitLab defaults target_branch to the project's default branch,
                // so we only pin it when a base is explicitly provided.
                let mut url = format!(
                    "https://{}/{}/-/merge_requests/new?merge_request[source_branch]={}",
                    self.host.name(),
                    self.slug(),
                    encoded_bookmark
                );
                if !base.is_empty() {
                    url.push_str("&merge_request[target_branch]=");
                    url.push_str(&encode_pull_request_ref(base));
                }
                url
            }
        }
    }

    fn slug(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }
}

impl RepoHost {
    fn from_name(name: &str) -> Option<Self> {
        if name.eq_ignore_ascii_case(GITHUB_HOST) {
            Some(Self::GitHub)
        } else if name.eq_ignore_ascii_case(CODEBERG_HOST) {
            Some(Self::Codeberg)
        } else if name.eq_ignore_ascii_case(GITLAB_HOST) {
            Some(Self::GitLab)
        } else {
            None
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::GitHub => GITHUB_HOST,
            Self::Codeberg => CODEBERG_HOST,
            Self::GitLab => GITLAB_HOST,
        }
    }

    pub(crate) fn display_name(self) -> &'static str {
        match self {
            Self::GitHub => "GitHub",
            Self::Codeberg => "Codeberg",
            Self::GitLab => "GitLab",
        }
    }
}

fn parse_owner_repo(path: &str) -> Option<(String, String)> {
    let mut parts = path.trim_matches('/').split('/').filter(|s| !s.is_empty());
    let owner = parts.next()?;
    let repo = parts.next()?;
    let repo = repo.strip_suffix(".git").unwrap_or(repo);
    if owner.is_empty() || repo.is_empty() || parts.next().is_some() {
        return None;
    }
    Some((owner.to_owned(), repo.to_owned()))
}

fn encode_pull_request_ref(s: &str) -> String {
    utf8_percent_encode(s, PULL_REQUEST_REF_ENCODE_SET).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_github_scp_remote() {
        let remote = HostedRepo::parse("git@github.com:hewigovens/jayjay.git").unwrap();
        assert_eq!(remote.host, RepoHost::GitHub);
        assert_eq!(remote.host.display_name(), "GitHub");
        assert_eq!(remote.slug(), "hewigovens/jayjay");
        assert_eq!(remote.host, RepoHost::GitHub);
        assert_eq!(
            remote.pull_request_open_url("feat/foo", "main"),
            "https://github.com/hewigovens/jayjay/pull/new/feat/foo"
        );
    }

    #[test]
    fn parses_codeberg_https_remote() {
        let remote = HostedRepo::parse("https://codeberg.org/hewigovens/jayjay.git\n").unwrap();
        assert_eq!(remote.host, RepoHost::Codeberg);
        assert_eq!(remote.host.display_name(), "Codeberg");
        assert_eq!(remote.slug(), "hewigovens/jayjay");
        assert_eq!(remote.host, RepoHost::Codeberg);
        assert_eq!(
            remote.pull_request_open_url("feat/foo", "master"),
            "https://codeberg.org/hewigovens/jayjay/compare/master...feat/foo"
        );
    }

    #[test]
    fn parses_gitlab_https_remote() {
        let remote = HostedRepo::parse("https://gitlab.com/hewigovens/jayjay.git").unwrap();
        assert_eq!(remote.host, RepoHost::GitLab);
        assert_eq!(remote.host.display_name(), "GitLab");
        assert_eq!(remote.slug(), "hewigovens/jayjay");
        assert_eq!(
            remote.pull_request_open_url("feat/foo", ""),
            "https://gitlab.com/hewigovens/jayjay/-/merge_requests/new?merge_request[source_branch]=feat/foo"
        );
        assert_eq!(
            remote.pull_request_open_url("feat/foo", "main"),
            "https://gitlab.com/hewigovens/jayjay/-/merge_requests/new?merge_request[source_branch]=feat/foo&merge_request[target_branch]=main"
        );
    }

    #[test]
    fn parses_gitlab_scp_remote() {
        let remote = HostedRepo::parse("git@gitlab.com:hewigovens/jayjay.git").unwrap();
        assert_eq!(remote.host, RepoHost::GitLab);
        assert_eq!(remote.slug(), "hewigovens/jayjay");
    }

    #[test]
    fn parses_codeberg_ssh_url_remote() {
        let remote = HostedRepo::parse("ssh://git@codeberg.org/hewig/jj-test.git").unwrap();
        assert_eq!(remote.host, RepoHost::Codeberg);
        assert_eq!(remote.slug(), "hewig/jj-test");
    }

    #[test]
    fn codeberg_uses_main_when_no_base_is_provided() {
        let remote = HostedRepo::parse("https://codeberg.org/hewigovens/jayjay.git").unwrap();
        assert_eq!(
            remote.pull_request_open_url("feat/foo", ""),
            "https://codeberg.org/hewigovens/jayjay/compare/main...feat/foo"
        );
    }

    #[test]
    fn encode_pull_request_ref_escapes_specials_and_preserves_slash() {
        let remote = HostedRepo::parse("https://codeberg.org/hewigovens/jayjay.git").unwrap();
        assert_eq!(
            remote.pull_request_open_url("weird#name?yes", "release/1.0"),
            "https://codeberg.org/hewigovens/jayjay/compare/release/1.0...weird%23name%3Fyes"
        );
    }

    #[test]
    fn rejects_unsupported_and_malformed_remotes() {
        for raw in [
            "https://github.com.evil.org/hewigovens/jayjay",
            "https://evilgithub.com/foo/bar",
            "https://codeberg.org.evil.org/hewigovens/jayjay",
            "https://evilcodeberg.org/foo/bar",
            "https://gitlab.com.evil.org/hewigovens/jayjay",
            "https://evilgitlab.com/foo/bar",
            "https://github.com/lonely",
            "https://github.com/hewigovens/jayjay/extra",
            "/Users/hewig/workspace/h/jayjay",
            "",
        ] {
            assert_eq!(HostedRepo::parse(raw), None);
        }
    }
}
