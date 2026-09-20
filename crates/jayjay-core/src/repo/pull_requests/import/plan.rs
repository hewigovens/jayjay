use crate::repo::git::{GitRemote, free_remote_name};
use crate::repo::hosted_repo::HostedRepo;
use crate::types::PrState;

pub(super) struct ResolvedPullRequest {
    pub(super) number: u32,
    pub(super) title: String,
    pub(super) state: PrState,
    pub(super) url: String,
    pub(super) head_branch: String,
    pub(super) head_commit_id: String,
    pub(super) head: PrHeadRepo,
}

impl ResolvedPullRequest {
    pub(super) fn short_head(&self) -> &str {
        self.head_commit_id.get(..8).unwrap_or(&self.head_commit_id)
    }

    pub(super) fn has_head(&self, commit_id: &str) -> bool {
        self.head_commit_id.eq_ignore_ascii_case(commit_id)
    }

    /// None when the host answered for another pull request or without a full head commit and branch.
    pub(super) fn checked(self, number: u32) -> Option<Self> {
        let has_full_head = self.head_commit_id.len() == 40
            && self.head_commit_id.bytes().all(|b| b.is_ascii_hexdigit());
        (self.number == number && has_full_head && !self.head_branch.is_empty()).then_some(self)
    }
}

pub(super) enum PrHeadRepo {
    SameRepository,
    Fork(HostedRepo),
}

pub(super) struct PullRequestImportPlan {
    pub(super) base: HostedRepo,
    pub(super) resolved: ResolvedPullRequest,
    pub(super) remote: RemoteChoice,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum RemoteChoice {
    Reuse { name: String, url: String },
    Add { name: String, url: String },
}

impl RemoteChoice {
    pub(super) fn name(&self) -> &str {
        match self {
            Self::Reuse { name, .. } | Self::Add { name, .. } => name,
        }
    }

    pub(super) fn url(&self) -> &str {
        match self {
            Self::Reuse { url, .. } | Self::Add { url, .. } => url,
        }
    }

    pub(super) fn exists(&self) -> bool {
        matches!(self, Self::Reuse { .. })
    }
}

pub(super) fn choose_remote(
    remotes: &[GitRemote],
    preferred_name: &str,
    head_url: &str,
) -> RemoteChoice {
    match remotes.iter().find(|remote| remote.points_at(head_url)) {
        Some(remote) => RemoteChoice::Reuse {
            name: remote.name.clone(),
            url: remote.fetch_url.clone(),
        },
        None => RemoteChoice::Add {
            name: free_remote_name(remotes, preferred_name),
            url: head_url.to_owned(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remotes(pairs: &[(&str, &str)]) -> Vec<GitRemote> {
        pairs
            .iter()
            .map(|(name, url)| GitRemote {
                name: name.to_string(),
                fetch_url: url.to_string(),
            })
            .collect()
    }

    #[test]
    fn reuses_remote_with_matching_url_across_schemes() {
        let existing = remotes(&[
            ("origin", "https://github.com/hewigovens/jayjay.git"),
            ("alice", "git@github.com:alice/jayjay.git"),
        ]);
        let choice = choose_remote(&existing, "alice", "https://github.com/alice/jayjay.git");
        assert_eq!(
            choice,
            RemoteChoice::Reuse {
                name: "alice".into(),
                url: "git@github.com:alice/jayjay.git".into(),
            }
        );
    }

    #[test]
    fn adds_remote_with_owner_name_when_free() {
        let existing = remotes(&[("origin", "https://github.com/hewigovens/jayjay.git")]);
        let choice = choose_remote(&existing, "alice", "https://github.com/alice/jayjay.git");
        assert_eq!(
            choice,
            RemoteChoice::Add {
                name: "alice".into(),
                url: "https://github.com/alice/jayjay.git".into(),
            }
        );
    }

    #[test]
    fn picks_stable_alternative_when_preferred_name_points_elsewhere() {
        let existing = remotes(&[
            ("origin", "https://github.com/hewigovens/jayjay.git"),
            ("alice", "https://github.com/someone/else.git"),
        ]);
        let choice = choose_remote(&existing, "alice", "https://github.com/alice/jayjay.git");
        assert_eq!(
            choice,
            RemoteChoice::Add {
                name: "alice-2".into(),
                url: "https://github.com/alice/jayjay.git".into(),
            }
        );
    }

    #[test]
    fn matches_local_path_remotes_verbatim() {
        let existing = remotes(&[("fork", "/tmp/fork.git")]);
        let choice = choose_remote(&existing, "fork", "/tmp/fork.git");
        assert!(matches!(choice, RemoteChoice::Reuse { .. }));
    }
}
