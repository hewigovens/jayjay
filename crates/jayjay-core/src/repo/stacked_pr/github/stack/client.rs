use std::fmt;
use std::process::Output;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::repo::{Repo, environment::gh_binary, hosted_repo::HostedRepo};

pub(super) struct Client<'a> {
    repo: &'a Repo,
    endpoint: String,
}

impl<'a> Client<'a> {
    pub(super) fn new(repo: &'a Repo, remote: &HostedRepo) -> Self {
        Self {
            repo,
            endpoint: format!("repos/{}/{}/stacks", remote.owner, remote.repo),
        }
    }

    pub(super) fn find_for_pr(&self, pr_number: u32) -> Result<Option<Stack>, ApiError> {
        let stacks: Vec<Stack> =
            self.request("GET", &self.endpoint, [format!("pull_request={pr_number}")])?;
        Ok(stacks.into_iter().next())
    }

    pub(super) fn create(&self, pr_numbers: &[u32]) -> Result<Stack, ApiError> {
        self.request("POST", &self.endpoint, pull_request_fields(pr_numbers))
    }

    pub(super) fn add(&self, stack_number: u32, pr_numbers: &[u32]) -> Result<Stack, ApiError> {
        self.request(
            "POST",
            &format!("{}/{stack_number}/add", self.endpoint),
            pull_request_fields(pr_numbers),
        )
    }

    fn request<T: DeserializeOwned>(
        &self,
        method: &str,
        endpoint: &str,
        fields: impl IntoIterator<Item = String>,
    ) -> Result<T, ApiError> {
        let mut args = vec![
            "api".to_owned(),
            "--method".to_owned(),
            method.to_owned(),
            endpoint.to_owned(),
        ];
        for field in fields {
            args.push("-F".to_owned());
            args.push(field);
        }
        let args = args.iter().map(String::as_str).collect::<Vec<_>>();
        let output = self
            .repo
            .command_output(&gh_binary(), &args, "gh api")
            .map_err(|error| ApiError::new(None, error.to_string()))?;
        if !output.status.success() {
            return Err(ApiError::from_output(&output));
        }
        serde_json::from_str(&Repo::stdout_text(&output))
            .map_err(|error| ApiError::new(None, format!("invalid GitHub response: {error}")))
    }
}

fn pull_request_fields(pr_numbers: &[u32]) -> impl Iterator<Item = String> + '_ {
    pr_numbers
        .iter()
        .map(|number| format!("pull_requests[]={number}"))
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

#[derive(Debug)]
pub(super) struct ApiError {
    status: Option<u16>,
    message: String,
}

impl ApiError {
    pub(super) fn new(status: Option<u16>, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub(super) fn status(&self) -> Option<u16> {
        self.status
    }

    fn from_output(output: &Output) -> Self {
        Self::from_text(&Repo::stdout_text(output), &Repo::stderr_text(output))
    }

    fn from_text(stdout: &str, stderr: &str) -> Self {
        #[derive(Deserialize)]
        struct ErrorBody {
            message: String,
            status: Option<String>,
        }

        let body = serde_json::from_str::<ErrorBody>(stdout).ok();
        let stderr = stderr.trim();
        let message = body
            .as_ref()
            .map(|body| body.message.as_str())
            .filter(|message| !message.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| (!stderr.is_empty()).then(|| stderr.to_owned()))
            .unwrap_or_else(|| "gh api failed".to_owned());
        let status = stderr
            .split_once("(HTTP ")
            .and_then(|(_, rest)| rest.split_once(')'))
            .and_then(|(status, _)| status.parse().ok());
        let status = status.or_else(|| {
            body.as_ref()
                .and_then(|body| body.status.as_deref())
                .and_then(|status| status.parse().ok())
        });
        Self::new(status, message)
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

#[cfg(test)]
mod tests;
