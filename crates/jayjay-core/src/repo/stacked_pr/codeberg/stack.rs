use jayjay_network::Fetch;

use super::super::forge::{ForgeTarget, created_with_number, failed, non_empty_or};
use super::section::{with_existing_section, with_stack_section};
use crate::repo::hosted_repo::HostedRepo;
use crate::repo::pull_requests::codeberg::{
    Api, ApiError, CodebergPrResponse, CreatePull, EditPull, PageScan, PullFilter,
};
use crate::types::{JayError, JayResult, StackLayerOutcome, SubmittedLayer};

pub(crate) struct CodebergStack<'a, F: Fetch> {
    api: Api<F>,
    remote: &'a HostedRepo,
    has_token: bool,
}

impl<'a, F: Fetch> CodebergStack<'a, F> {
    pub(crate) fn new(fetch: F, remote: &'a HostedRepo, token: Option<String>) -> Self {
        Self {
            has_token: token
                .as_deref()
                .is_some_and(|token| !token.trim().is_empty()),
            api: Api::new(fetch, remote, token),
            remote,
        }
    }

    pub(crate) fn preflight(&self) -> JayResult<String> {
        if !self.has_token {
            return Err(JayError::internal(
                "Codeberg stacked pull requests need a token; set the CODEBERG_TOKEN environment variable (or FORGEJO_TOKEN) and retry.",
            ));
        }
        let repository = self
            .api
            .repository()
            .map_err(|error| self.preflight_error(error))?;
        if !repository.push() {
            return Err(JayError::internal(format!(
                "The Codeberg token cannot push to {}; grant the token write access to the repository and retry.",
                self.remote.slug()
            )));
        }
        match repository.default_branch() {
            "" => Err(JayError::internal(format!(
                "Codeberg does not report a default branch for {}.",
                self.remote.slug()
            ))),
            branch => Ok(branch.to_owned()),
        }
    }

    pub(crate) fn submit_layer(&self, target: &ForgeTarget) -> SubmittedLayer {
        let scan = self.api.find_pull(PullFilter::Open, |pr| {
            pr.is_stack_layer_of(self.remote, &target.bookmark)
        });
        match scan {
            PageScan::Found(pull) => self.update_layer(target, &pull),
            PageScan::Exhausted => self.create_layer(target),
            // Creating without a conclusive lookup could duplicate an existing PR.
            PageScan::Failed | PageScan::Capped => failed(
                target,
                "Could not check for an existing pull request; retry.".to_owned(),
            ),
        }
    }

    pub(crate) fn link(
        &self,
        targets: &[ForgeTarget],
        layers: &[SubmittedLayer],
    ) -> Result<(), String> {
        if layers.len() < 2 {
            return Ok(());
        }
        // A section with a missing layer would misstate which PR each one builds on.
        if layers
            .iter()
            .any(|layer| layer.outcome == StackLayerOutcome::Failed || layer.pr_number == 0)
        {
            return Err(
                "Stack links were not updated because a layer failed; submit again to refresh them."
                    .to_owned(),
            );
        }
        let numbers: Vec<u32> = layers.iter().map(|layer| layer.pr_number).collect();
        let notes: Vec<String> = targets
            .iter()
            .zip(layers)
            .filter_map(|(target, layer)| {
                let title = non_empty_or(&target.title, &target.bookmark);
                let edit = EditPull {
                    base: layer.base.clone(),
                    body: with_stack_section(
                        &non_empty_or(&target.body, &title),
                        layer.pr_number,
                        &numbers,
                    ),
                    title,
                };
                self.api
                    .edit_pull(layer.pr_number, &edit)
                    .err()
                    .map(|error| {
                        format!(
                            "Could not add the stack section to PR #{}: {error}.",
                            layer.pr_number
                        )
                    })
            })
            .collect();
        if notes.is_empty() {
            Ok(())
        } else {
            Err(notes.join("\n"))
        }
    }

    fn create_layer(&self, target: &ForgeTarget) -> SubmittedLayer {
        let title = non_empty_or(&target.title, &target.bookmark);
        let pull = CreatePull {
            head: target.bookmark.clone(),
            base: target.base.clone(),
            body: non_empty_or(&target.body, &title),
            title: title.clone(),
        };
        match self.api.create_pull(&pull) {
            Ok(created) => created_with_number(target, title, created.number(), created.web_url()),
            Err(error) => failed(target, error.to_string()),
        }
    }

    fn update_layer(&self, target: &ForgeTarget, pull: &CodebergPrResponse) -> SubmittedLayer {
        let title = non_empty_or(&target.title, &target.bookmark);
        let edit = EditPull {
            base: target.base.clone(),
            body: with_existing_section(&non_empty_or(&target.body, &title), pull.body()),
            title: title.clone(),
        };
        match self.api.edit_pull(pull.number(), &edit) {
            Ok(()) => SubmittedLayer {
                bookmark: target.bookmark.clone(),
                base: target.base.clone(),
                title,
                outcome: StackLayerOutcome::Updated,
                pr_number: pull.number(),
                pr_url: pull.web_url(),
                detail: format!("PR #{} → base {}", pull.number(), target.base),
            },
            Err(error) => failed(target, error.to_string()),
        }
    }

    fn preflight_error(&self, error: ApiError) -> JayError {
        let slug = self.remote.slug();
        JayError::internal(match error.status() {
            Some(401) => {
                "The Codeberg token was rejected; check that CODEBERG_TOKEN is current and retry."
                    .to_owned()
            }
            Some(404) => format!(
                "Codeberg repository {slug} was not found or the token cannot see it; check the remote and the token's access."
            ),
            _ => format!("Could not check the Codeberg repository {slug}: {error}."),
        })
    }
}
