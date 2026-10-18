use jayjay_network::{Fetch, Method};
use serde::Deserialize;
use serde_json::json;

use crate::repo::forge_auth::TokenProvider;
use crate::repo::pull_requests::ApiError;
use crate::repo::pull_requests::github::Api;

pub(super) struct Client<'a, F: Fetch, T: TokenProvider> {
    api: &'a Api<F, T>,
}

impl<'a, F: Fetch, T: TokenProvider> Client<'a, F, T> {
    pub(super) fn new(api: &'a Api<F, T>) -> Self {
        Self { api }
    }

    pub(super) fn find_for_pr(&self, pr_number: u32) -> Result<Option<Stack>, ApiError> {
        let stacks: Vec<Stack> = self.api.client().get(
            &self
                .api
                .repo_path(&format!("stacks?pull_request={pr_number}")),
        )?;
        Ok(stacks.into_iter().next())
    }

    pub(super) fn create(&self, pr_numbers: &[u32]) -> Result<Stack, ApiError> {
        self.api.client().send(
            Method::Post,
            &self.api.repo_path("stacks"),
            &json!({ "pull_requests": pr_numbers }),
        )
    }

    pub(super) fn add(&self, stack_number: u32, pr_numbers: &[u32]) -> Result<Stack, ApiError> {
        self.api.client().send(
            Method::Post,
            &self.api.repo_path(&format!("stacks/{stack_number}/add")),
            &json!({ "pull_requests": pr_numbers }),
        )
    }
}

#[derive(Deserialize)]
pub(super) struct Stack {
    number: u32,
    pull_requests: Vec<StackPullRequest>,
}

impl Stack {
    pub(super) fn number(&self) -> u32 {
        self.number
    }

    pub(super) fn pr_numbers(&self) -> Vec<u32> {
        self.pull_requests.iter().map(|pr| pr.number).collect()
    }

    #[cfg(test)]
    pub(super) fn from_pull_request_numbers(number: u32, pull_requests: &[u32]) -> Self {
        Self {
            number,
            pull_requests: pull_requests
                .iter()
                .map(|number| StackPullRequest { number: *number })
                .collect(),
        }
    }
}

#[derive(Deserialize)]
struct StackPullRequest {
    number: u32,
}
