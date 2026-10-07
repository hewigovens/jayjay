use crate::repo::Repo;
use crate::repo::hosted_repo::{HostedRepo, RepoHost};
use crate::types::*;

use super::cursor;
use super::forge::ForgeTarget;
use super::github;
use super::gitlab;
use super::naming::is_valid_bookmark_name;
use super::validation::validate_stack_changes;

impl Repo {
    pub fn submit_stack(&self, layers: Vec<SubmitStackLayer>) -> JayResult<StackedPrResult> {
        if layers.is_empty() {
            return Err(JayError::Internal {
                message: "No changes to submit.".to_owned(),
            });
        }
        if let Some(bad) = layers.iter().find(|l| !is_valid_bookmark_name(&l.bookmark)) {
            return Err(JayError::Internal {
                message: format!("\"{}\" is not a valid branch name.", bad.bookmark),
            });
        }

        // Two layers sharing a bookmark would move it twice and mis-head the PRs.
        let mut seen = std::collections::HashSet::new();
        if let Some(dup) = layers.iter().find(|l| !seen.insert(l.bookmark.as_str())) {
            return Err(JayError::Internal {
                message: format!(
                    "Bookmark \"{}\" is used by more than one change.",
                    dup.bookmark
                ),
            });
        }

        // The panel is only a preview; re-resolve every change before the first bookmark move.
        self.reload()?;
        self.validate_stack(&layers)?;

        let remote = self
            .git_remote_url()
            .ok()
            .and_then(|url| HostedRepo::parse(&url));
        let remote = match remote {
            Some(remote)
                if matches!(
                    remote.host,
                    RepoHost::GitHub | RepoHost::GitLab | RepoHost::Cursor
                ) =>
            {
                remote
            }
            _ => {
                return Err(JayError::Internal {
                    message: "Stacked PRs support GitHub, GitLab, and Cursor remotes.".to_owned(),
                });
            }
        };
        let host = remote.host;

        // Preflight the forge CLI first, or a failure leaves dangling remote branches and moved bookmarks.
        let base_bookmark = match host {
            RepoHost::GitLab => {
                gitlab::preflight(self)?;
                self.default_pull_request_base()
            }
            RepoHost::Cursor => cursor::preflight(self)?,
            _ => {
                github::preflight(self)?;
                self.default_pull_request_base()
            }
        };

        let mut targets: Vec<ForgeTarget> = Vec::with_capacity(layers.len());
        let write = self.write_guard()?;
        // Forge authentication can take a while; validate again against the head the lock just reloaded.
        self.validate_stack(&layers)?;
        for (i, layer) in layers.iter().enumerate() {
            self.move_bookmark(&layer.bookmark, &layer.change_id)?;
            let base = if i == 0 {
                base_bookmark.clone()
            } else {
                layers[i - 1].bookmark.clone()
            };
            targets.push(ForgeTarget {
                bookmark: layer.bookmark.clone(),
                base,
                title: layer.title.clone(),
                body: layer.body.clone(),
            });
        }

        drop(write);

        // Push the whole set first so every PR base and head exists.
        let names: Vec<&str> = targets.iter().map(|t| t.bookmark.as_str()).collect();
        let mut message = self.git_push_bookmarks(&names)?;

        let layers: Vec<_> = targets
            .iter()
            .map(|target| match host {
                RepoHost::GitLab => gitlab::create_or_update_mr(self, target),
                RepoHost::Cursor => cursor::create_or_update_pr(self, target),
                _ => github::create_or_update_pr(self, target),
            })
            .collect();

        let mut native_stack_linked = false;
        if host == RepoHost::GitHub
            && let Some(native_outcome) = github::reconcile_stack(self, &remote, &layers)
        {
            native_stack_linked = native_outcome.is_linked();
            if !message.is_empty() {
                message.push('\n');
            }
            message.push_str(&native_outcome.into_message());
        }

        let open_urls = result_open_urls(&layers, host, native_stack_linked);
        Ok(StackedPrResult {
            layers,
            message,
            open_urls,
        })
    }

    fn validate_stack(&self, layers: &[SubmitStackLayer]) -> JayResult<()> {
        validate_stack_changes(&self.resolve_stack_changes(layers)?)?;
        self.ensure_bookmarks_unclaimed(layers)
    }

    // An edited name may already belong to another change (worst case: trunk); reject instead of silently retargeting it.
    fn ensure_bookmarks_unclaimed(&self, layers: &[SubmitStackLayer]) -> JayResult<()> {
        let bookmarks = self.list_bookmarks()?;
        if let Some((layer, existing)) = layers.iter().find_map(|layer| {
            bookmarks
                .iter()
                .find(|bookmark| {
                    bookmark.name == layer.bookmark
                        && !self.bookmark_targets_change(bookmark, &layer.change_id)
                })
                .map(|bookmark| (layer, bookmark))
        }) {
            let owner = if existing.is_conflicted {
                "a conflicted change".to_owned()
            } else {
                let origin_owner = self
                    .remote_bookmark_change_id(&existing.name, "origin")
                    .filter(|change| !change.is_empty() && change != &layer.change_id);
                let local_owner = (existing.has_local_target
                    && !existing.is_deleted
                    && existing.change_id.as_str() != layer.change_id)
                    .then(|| existing.change_id.id.clone());
                origin_owner.or(local_owner).map_or_else(
                    || "another local or origin change".to_owned(),
                    |change| format!("change {change}"),
                )
            };
            return Err(JayError::Internal {
                message: format!(
                    "Bookmark \"{}\" already belongs to {owner}; choose a different bookmark for change {}.",
                    layer.bookmark, layer.change_id
                ),
            });
        }
        Ok(())
    }

    fn resolve_stack_changes(&self, layers: &[SubmitStackLayer]) -> JayResult<Vec<ChangeInfo>> {
        layers
            .iter()
            .map(|layer| {
                let mut matches = self.log(&layer.change_id)?;
                if matches.len() != 1 || matches[0].change_id.id != layer.change_id {
                    return Err(JayError::Internal {
                        message: "The stack changed since preview. Refresh it before submitting."
                            .to_owned(),
                    });
                }
                Ok(matches.pop().expect("exactly one stack change"))
            })
            .collect()
    }

    fn bookmark_targets_change(&self, bookmark: &BookmarkInfo, change_id: &str) -> bool {
        if bookmark.is_conflicted {
            return false;
        }
        let local_active = bookmark.has_local_target && !bookmark.is_deleted;
        if local_active && bookmark.change_id.as_str() != change_id {
            return false;
        }
        let origin_change = self.remote_bookmark_change_id(&bookmark.name, "origin");
        if origin_change
            .as_deref()
            .is_some_and(|remote_change| remote_change != change_id)
        {
            return false;
        }
        local_active || origin_change.as_deref() == Some(change_id)
    }
}

pub(super) fn result_open_urls(
    layers: &[SubmittedLayer],
    host: RepoHost,
    native_stack_linked: bool,
) -> Vec<String> {
    if host == RepoHost::GitLab {
        return layers
            .iter()
            .rev()
            .map(|layer| layer.pr_url.as_str())
            .find(|url| !url.is_empty())
            .map(|url| vec![url.to_owned()])
            .unwrap_or_default();
    }
    if native_stack_linked
        && let Some(top_url) = layers
            .last()
            .map(|layer| layer.pr_url.as_str())
            .filter(|url| !url.is_empty())
    {
        return vec![top_url.to_owned()];
    }
    layers
        .iter()
        .map(|layer| layer.pr_url.as_str())
        .filter(|url| !url.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
