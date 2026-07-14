mod forge;
mod github;
mod gitlab;
mod naming;

pub use naming::is_valid_bookmark_name;

use super::Repo;
use super::hosted_repo::{HostedRepo, RepoHost};
use crate::types::*;
use forge::ForgeTarget;

impl Repo {
    /// Detect and validate the linear stack `base..tip` (base is usually
    /// `trunk()`), computing each layer's bookmark and dependent base. No side
    /// effects — drives the preview.
    pub fn detect_stack(&self, base_rev: &str, tip_rev: &str) -> CoreResult<Stack> {
        let mut changes = self.log(&format!("{base_rev}..{tip_rev}"))?;
        if changes.is_empty() {
            return Err(CoreError::Internal {
                message: "No mutable changes above trunk to stack.".to_owned(),
            });
        }
        changes.reverse(); // bottom → top
        validate_stack_changes(&changes)?;

        let base_bookmark = self.default_pull_request_base();
        let mut layers: Vec<StackLayer> = Vec::with_capacity(changes.len());
        for (i, change) in changes.iter().enumerate() {
            let short_len = (change.change_id.short_len as usize).min(change.change_id.id.len());
            let change_id_short = change.change_id.id[..short_len].to_owned();
            let existing = change.bookmarks.first().cloned();
            let bookmark = existing.clone().unwrap_or_else(|| {
                naming::bookmark_name(
                    &change.description,
                    &change.change_id.id,
                    change.change_id.short_len,
                )
            });
            let base = if i == 0 {
                base_bookmark.clone()
            } else {
                layers[i - 1].bookmark.clone()
            };
            layers.push(StackLayer {
                change_id: change.change_id.id.clone(),
                commit_id: change.commit_id.id.clone(),
                title: naming::first_line(&change.description),
                body: naming::body(&change.description),
                bookmark,
                base,
                bookmark_existed: existing.is_some(),
                change_id_short,
            });
        }
        Ok(Stack {
            layers,
            base_bookmark,
        })
    }

    /// Assign + push the per-change bookmarks chosen by the UI (possibly edited or
    /// AI-generated), then create/update PRs bottom→top with dependent bases.
    /// Idempotent: bookmarks anchor on change-id, and re-running updates the PRs.
    pub fn submit_stack(&self, layers: Vec<SubmitStackLayer>) -> CoreResult<StackedPrResult> {
        if layers.is_empty() {
            return Err(CoreError::Internal {
                message: "No changes to submit.".to_owned(),
            });
        }
        // Reject bad branch names up front so we never half-assign local bookmarks
        // and then fail at push time.
        if let Some(bad) = layers.iter().find(|l| !is_valid_bookmark_name(&l.bookmark)) {
            return Err(CoreError::Internal {
                message: format!("\"{}\" is not a valid branch name.", bad.bookmark),
            });
        }

        // Two layers sharing a bookmark would move it twice and compute the upper
        // layer's base from the duplicate name, leaving one bookmark at the top and
        // mis-heading the PRs. Reject before any `move_bookmark` side effects.
        let mut seen = std::collections::HashSet::new();
        if let Some(dup) = layers.iter().find(|l| !seen.insert(l.bookmark.as_str())) {
            return Err(CoreError::Internal {
                message: format!(
                    "Bookmark \"{}\" is used by more than one change.",
                    dup.bookmark
                ),
            });
        }

        // The panel is only a preview. Reload and resolve every change again before the first bookmark move so an external abandon, divergence, or reparent cannot leave a partially submitted stack.
        self.reload()?;
        let current_changes = self.resolve_stack_changes(&layers)?;
        validate_stack_changes(&current_changes)?;

        // An edited name may already belong to another local or remote change (most dangerously, the trunk bookmark), so reject the whole plan before forge preflight or any bookmark moves instead of silently retargeting it.
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
            return Err(CoreError::Internal {
                message: format!(
                    "Bookmark \"{}\" already belongs to {owner}; choose a different bookmark for change {}.",
                    layer.bookmark, layer.change_id
                ),
            });
        }

        // Dependent bases work the same on GitHub (`gh`) and GitLab (`glab`).
        let host = self
            .git_remote_url()
            .ok()
            .and_then(|url| HostedRepo::parse(&url))
            .map(|remote| remote.host);
        let host = match host {
            Some(host @ (RepoHost::GitHub | RepoHost::GitLab)) => host,
            _ => {
                return Err(CoreError::Internal {
                    message: "Stacked PRs support GitHub and GitLab remotes.".to_owned(),
                });
            }
        };

        // Prove the forge CLI is installed and authenticated before any local
        // bookmark move or push, so a missing/misconfigured CLI fails up front
        // instead of leaving dangling remote branches and moved bookmarks.
        match host {
            RepoHost::GitLab => gitlab::preflight(self)?,
            _ => github::preflight(self)?,
        }

        // Point each bookmark at its change (create-or-move) and compute the
        // dependent base from the submitted order: bottom → trunk, others → below.
        let base_bookmark = self.default_pull_request_base();
        let mut targets: Vec<ForgeTarget> = Vec::with_capacity(layers.len());
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

        // Push the whole set first so every PR base/head exists, then create.
        let names: Vec<&str> = targets.iter().map(|t| t.bookmark.as_str()).collect();
        let message = self.git_push_bookmarks(&names)?;

        let layers = targets
            .iter()
            .map(|target| match host {
                RepoHost::GitLab => gitlab::create_or_update_mr(self, target),
                _ => github::create_or_update_pr(self, target),
            })
            .collect();

        Ok(StackedPrResult { layers, message })
    }

    fn resolve_stack_changes(&self, layers: &[SubmitStackLayer]) -> CoreResult<Vec<ChangeInfo>> {
        layers
            .iter()
            .map(|layer| {
                let mut matches = self.log(&layer.change_id)?;
                if matches.len() != 1 || matches[0].change_id.id != layer.change_id {
                    return Err(CoreError::Internal {
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

fn validate_stack_changes(changes: &[ChangeInfo]) -> CoreResult<()> {
    for (i, change) in changes.iter().enumerate() {
        if change.is_immutable {
            return Err(CoreError::Internal {
                message: "The stack contains an immutable change.".to_owned(),
            });
        }
        if change.is_divergent {
            return Err(CoreError::Internal {
                message: "The stack contains a divergent change. Resolve the divergence (abandon the duplicate) before submitting."
                    .to_owned(),
            });
        }
        if change.parents.len() != 1 || (i > 0 && change.parents[0] != changes[i - 1].commit_id.id)
        {
            return Err(CoreError::Internal {
                message: "The stack must be linear (no merges).".to_owned(),
            });
        }
    }
    Ok(())
}
