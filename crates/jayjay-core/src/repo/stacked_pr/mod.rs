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

        let base_bookmark = self.default_pull_request_base();
        let mut layers: Vec<StackLayer> = Vec::with_capacity(changes.len());
        for (i, change) in changes.iter().enumerate() {
            if change.is_immutable {
                return Err(CoreError::Internal {
                    message: "The stack contains an immutable change.".to_owned(),
                });
            }
            if change.parents.len() != 1
                || (i > 0 && change.parents[0] != changes[i - 1].commit_id.id)
            {
                return Err(CoreError::Internal {
                    message: "The stack must be linear (no merges).".to_owned(),
                });
            }
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
}
