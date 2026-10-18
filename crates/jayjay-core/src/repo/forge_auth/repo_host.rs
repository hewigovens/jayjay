use std::process::Stdio;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use super::token::{ForgeToken, TokenProvider, TokenSource};
use crate::repo::environment::{command, wait_for_stdout};
use crate::repo::hosted_repo::RepoHost;

const CLI_TIMEOUT: Duration = Duration::from_secs(5);
const TOKEN_TTL: Duration = Duration::from_secs(300);
const REFRESH_BACKOFF: Duration = Duration::from_secs(60);

static TOKENS: Mutex<[TokenSlot; 4]> = Mutex::new([TokenSlot::EMPTY; 4]);

struct TokenSlot {
    /// A missing token is cached too, so an absent login isn't re-probed per request.
    resolved: Option<(Option<ForgeToken>, Instant)>,
    rejected: Option<(ForgeToken, Instant)>,
}

impl TokenSlot {
    const EMPTY: Self = Self {
        resolved: None,
        rejected: None,
    };

    fn cached(
        &mut self,
        now: Instant,
        resolve: impl FnOnce() -> Option<ForgeToken>,
    ) -> Option<ForgeToken> {
        if let Some((token, at)) = &self.resolved
            && now.duration_since(*at) < TOKEN_TTL
        {
            return token.clone();
        }
        let token = resolve();
        self.resolved = Some((token.clone(), now));
        token
    }

    fn refresh(
        &mut self,
        rejected: &ForgeToken,
        now: Instant,
        resolve: impl Fn() -> Option<ForgeToken>,
        refresh_login: impl FnOnce(),
    ) -> Option<ForgeToken> {
        let mut current = resolve();
        let recently_rejected = self.rejected.as_ref().is_some_and(|(token, at)| {
            token == rejected && now.duration_since(*at) < REFRESH_BACKOFF
        });
        let attempt = current.as_ref() == Some(rejected) && !recently_rejected;
        if attempt {
            refresh_login();
            current = resolve();
        }
        self.resolved = Some((current.clone(), now));
        let fresh = current.filter(|token| token != rejected);
        if attempt && fresh.is_none() {
            self.rejected = Some((rejected.clone(), now));
        }
        fresh
    }
}

struct CliLogin {
    binary: &'static str,
    print_token: &'static [&'static str],
    refresh: Option<&'static [&'static str]>,
}

impl RepoHost {
    /// Cursor goes through its own CLI.
    pub const TOKEN_HOSTS: [Self; 3] = [Self::GitHub, Self::GitLab, Self::Codeberg];

    pub fn token_source(self) -> Option<TokenSource> {
        self.token().map(|token| token.source)
    }

    pub(crate) fn sign_in_hint(self) -> &'static str {
        match self {
            Self::GitHub => "Run `gh auth login`, or set GH_TOKEN.",
            Self::GitLab => "Run `glab auth login`, or set GITLAB_TOKEN.",
            Self::Codeberg => "Set CODEBERG_TOKEN.",
            Self::Cursor => "Run `origin auth login`.",
        }
    }

    fn environment_variables(self) -> &'static [&'static str] {
        match self {
            Self::GitHub => &["GH_TOKEN", "GITHUB_TOKEN"],
            Self::GitLab => &["GITLAB_TOKEN"],
            Self::Codeberg => &["CODEBERG_TOKEN", "FORGEJO_TOKEN"],
            Self::Cursor => &[],
        }
    }

    fn cli_login(self) -> Option<CliLogin> {
        match self {
            Self::GitHub => Some(CliLogin {
                binary: "gh",
                print_token: &["auth", "token", "--hostname", "github.com"],
                refresh: None,
            }),
            Self::GitLab => Some(CliLogin {
                binary: "glab",
                print_token: &["config", "get", "token", "--host", "gitlab.com"],
                refresh: Some(&["auth", "status", "--hostname", "gitlab.com"]),
            }),
            Self::Codeberg | Self::Cursor => None,
        }
    }

    fn resolve_with(
        self,
        environment: impl Fn(&str) -> Option<String>,
        run: impl Fn(&str, &[&str]) -> Option<String>,
    ) -> Option<ForgeToken> {
        let non_empty = |secret: Option<String>| secret.filter(|secret| !secret.trim().is_empty());
        let from_environment = self.environment_variables().iter().find_map(|variable| {
            non_empty(environment(variable)).map(|secret| ForgeToken {
                secret,
                source: TokenSource::Environment(variable),
            })
        });
        from_environment.or_else(|| {
            let login = self.cli_login()?;
            non_empty(run(login.binary, login.print_token)).map(|secret| ForgeToken {
                secret,
                source: TokenSource::Cli(login.binary),
            })
        })
    }

    fn resolve(self) -> Option<ForgeToken> {
        self.resolve_with(|variable| std::env::var(variable).ok(), run_cli)
    }

    fn with_slot<T>(self, body: impl FnOnce(&mut TokenSlot) -> T) -> T {
        let mut tokens = TOKENS.lock().unwrap_or_else(PoisonError::into_inner);
        body(&mut tokens[self as usize])
    }
}

impl TokenProvider for RepoHost {
    fn token(&self) -> Option<ForgeToken> {
        self.with_slot(|slot| slot.cached(Instant::now(), || self.resolve()))
    }

    fn refreshed(&self, rejected: &ForgeToken) -> Option<ForgeToken> {
        self.with_slot(|slot| {
            slot.refresh(
                rejected,
                Instant::now(),
                || self.resolve(),
                || {
                    if let (TokenSource::Cli(_), Some(login)) = (rejected.source, self.cli_login())
                        && let Some(refresh) = login.refresh
                    {
                        run_cli(login.binary, refresh);
                    }
                },
            )
        })
    }

    fn forget(&self) {
        self.with_slot(|slot| *slot = TokenSlot::EMPTY);
    }
}

fn run_cli(binary: &str, args: &[&str]) -> Option<String> {
    let child = command(binary)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    wait_for_stdout(child, CLI_TIMEOUT)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve(
        host: RepoHost,
        environment: &[(&str, &str)],
        cli: Option<&str>,
    ) -> Option<ForgeToken> {
        host.resolve_with(
            |variable| {
                environment
                    .iter()
                    .find(|(name, _)| *name == variable)
                    .map(|(_, value)| (*value).to_owned())
            },
            |_, _| cli.map(str::to_owned),
        )
    }

    #[test]
    fn environment_wins_over_the_cli_login_in_declared_order() {
        let token = resolve(
            RepoHost::GitHub,
            &[("GITHUB_TOKEN", "b"), ("GH_TOKEN", "a")],
            Some("cli"),
        )
        .unwrap();
        assert_eq!(token.secret, "a");
        assert_eq!(token.source, TokenSource::Environment("GH_TOKEN"));

        let token = resolve(RepoHost::GitLab, &[], Some("cli")).unwrap();
        assert_eq!(token.source, TokenSource::Cli("glab"));
        assert_eq!(resolve(RepoHost::Codeberg, &[], Some("cli")), None);
        assert_eq!(resolve(RepoHost::GitHub, &[], Some("  ")), None);
        let token = resolve(RepoHost::GitHub, &[("GH_TOKEN", "")], Some("cli")).unwrap();
        assert_eq!(token.source, TokenSource::Cli("gh"));
    }

    fn token(secret: &str) -> Option<ForgeToken> {
        Some(ForgeToken {
            secret: secret.into(),
            source: TokenSource::Cli("glab"),
        })
    }

    #[test]
    fn a_lookup_is_reused_until_it_expires() {
        let start = Instant::now();
        let mut slot = TokenSlot::EMPTY;
        assert_eq!(slot.cached(start, || None), None);
        assert_eq!(
            slot.cached(start + Duration::from_secs(1), || panic!("cached")),
            None
        );
        let later = start + TOKEN_TTL;
        assert_eq!(slot.cached(later, || token("a")), token("a"));
    }

    #[test]
    fn a_rejected_token_switches_to_the_current_login_or_refreshes_once_per_backoff() {
        let start = Instant::now();
        let mut slot = TokenSlot::EMPTY;
        let old = token("old").unwrap();
        assert_eq!(
            slot.refresh(&old, start, || token("new"), || panic!("not needed")),
            token("new")
        );

        let revoked = token("new").unwrap();
        assert_eq!(slot.refresh(&revoked, start, || token("new"), || {}), None);
        let soon = start + Duration::from_secs(1);
        assert_eq!(
            slot.refresh(&revoked, soon, || token("new"), || panic!("throttled")),
            None
        );
        let later = start + REFRESH_BACKOFF;
        let refreshed = std::cell::Cell::new(false);
        let fresh = slot.refresh(
            &revoked,
            later,
            || {
                if refreshed.get() {
                    token("fixed")
                } else {
                    token("new")
                }
            },
            || refreshed.set(true),
        );
        assert_eq!(fresh, token("fixed"));
        assert_eq!(slot.cached(later, || panic!("cached")), token("fixed"));
    }

    #[test]
    fn debug_output_never_shows_the_secret() {
        let token = resolve(RepoHost::GitHub, &[("GH_TOKEN", "s3cret")], None).unwrap();
        assert!(!format!("{token:?}").contains("s3cret"));
    }
}
