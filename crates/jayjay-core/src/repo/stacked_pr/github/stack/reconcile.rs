use jayjay_network::Fetch;

use super::client::{Client, Stack};
use crate::repo::forge_auth::TokenProvider;
use crate::repo::pull_requests::ApiError;
use crate::repo::pull_requests::github::Api;
use crate::repo::stacked_pr::stack_link_outcome::StackLinkOutcome;
use crate::types::{StackLayerOutcome, SubmittedLayer};

const MAX_STACK_SIZE: usize = 100;

pub(crate) fn reconcile(
    api: &Api<impl Fetch, impl TokenProvider>,
    layers: &[SubmittedLayer],
) -> Option<StackLinkOutcome> {
    if layers.len() < 2 {
        return None;
    }
    if layers.len() > MAX_STACK_SIZE {
        return Some(StackLinkOutcome::Fallback(
            "Submitted the PRs as a dependent chain; GitHub native stacks support at most 100 PRs."
                .to_owned(),
        ));
    }
    if layers
        .iter()
        .any(|layer| layer.outcome == StackLayerOutcome::Failed || layer.pr_number == 0)
    {
        return Some(StackLinkOutcome::Fallback(
            "Submitted the available PRs as a dependent chain; native GitHub stack linking was skipped because one or more PR operations failed."
                .to_owned(),
        ));
    }

    let pr_numbers = layers
        .iter()
        .map(|layer| layer.pr_number)
        .collect::<Vec<_>>();
    Some(reconcile_remote(&Client::new(api), &pr_numbers))
}

fn reconcile_remote(
    client: &Client<impl Fetch, impl TokenProvider>,
    desired: &[u32],
) -> StackLinkOutcome {
    let mut retry_available = true;
    loop {
        let existing = match client.find_for_pr(desired[0]) {
            Ok(stack) => stack,
            Err(error) => return StackLinkOutcome::Fallback(fallback_message(&error)),
        };

        match plan_reconciliation(desired, existing.as_ref()) {
            ReconciliationPlan::Create(pull_requests) => {
                return match client.create(pull_requests) {
                    Ok(stack) => StackLinkOutcome::Linked(format!(
                        "Created native GitHub stack{} with {} PRs.",
                        stack_label(stack.number()),
                        pull_requests.len()
                    )),
                    Err(error) => StackLinkOutcome::Fallback(fallback_message(&error)),
                };
            }
            ReconciliationPlan::Current(stack_number) => {
                return StackLinkOutcome::Linked(format!(
                    "Native GitHub stack{} is already up to date.",
                    stack_label(stack_number)
                ));
            }
            ReconciliationPlan::Append {
                stack_number,
                pull_requests,
            } => match client.add(stack_number, pull_requests) {
                Ok(stack) => {
                    return StackLinkOutcome::Linked(format!(
                        "Updated native GitHub stack{} to {} PRs.",
                        stack_label(stack.number()),
                        desired.len()
                    ));
                }
                Err(error) => {
                    if retry_available && matches!(error.status(), Some(404 | 409)) {
                        retry_available = false;
                        continue;
                    }
                    return StackLinkOutcome::Fallback(fallback_message(&error));
                }
            },
            ReconciliationPlan::Diverged(stack_number) => {
                return StackLinkOutcome::Fallback(format!(
                    "Submitted the PRs, but native GitHub stack{} differs and cannot be updated automatically.",
                    stack_label(stack_number)
                ));
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ReconciliationPlan<'a> {
    Create(&'a [u32]),
    Current(u32),
    Append {
        stack_number: u32,
        pull_requests: &'a [u32],
    },
    Diverged(u32),
}

fn plan_reconciliation<'a>(desired: &'a [u32], existing: Option<&Stack>) -> ReconciliationPlan<'a> {
    let Some(existing) = existing else {
        return ReconciliationPlan::Create(desired);
    };
    let current = existing.pr_numbers();
    if current == desired {
        return ReconciliationPlan::Current(existing.number());
    }
    if let Some(pull_requests) = desired.strip_prefix(current.as_slice()) {
        return ReconciliationPlan::Append {
            stack_number: existing.number(),
            pull_requests,
        };
    }
    ReconciliationPlan::Diverged(existing.number())
}

fn fallback_message(error: &ApiError) -> String {
    if error.status() == Some(404) {
        return "Submitted the PRs as a dependent chain; native GitHub stacks are not enabled for this repository.".to_owned();
    }
    format!(
        "Submitted the PRs as a dependent chain; GitHub could not link the native stack: {error}."
    )
}

fn stack_label(number: u32) -> String {
    if number > 0 {
        format!(" #{number}")
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests;
