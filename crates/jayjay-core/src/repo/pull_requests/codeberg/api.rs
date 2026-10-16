use jayjay_network::{Auth, DEFAULT_TEXT_CAP, Fetch, Method, Request, encode_path_segment};
use serde::Serialize;
use serde::de::{DeserializeOwned, IgnoredAny};

use super::api_error::ApiError;
use super::page_scan::{PAGE_SIZE, PageScan, PullFilter, scan_pages};
use super::pull_request::{CodebergPrResponse, CreatePull, EditPull};
use super::repo::ForgejoRepo;
use super::status::CodebergCombinedStatus;
use crate::repo::hosted_repo::HostedRepo;

const API_URL: &str = "https://codeberg.org/api/v1";

pub(crate) fn codeberg_token() -> Option<String> {
    std::env::var("CODEBERG_TOKEN")
        .or_else(|_| std::env::var("FORGEJO_TOKEN"))
        .ok()
}

pub(crate) struct Api<F: Fetch> {
    fetch: F,
    repo_url: String,
    auth: Auth,
}

impl<F: Fetch> Api<F> {
    pub(crate) fn new(fetch: F, remote: &HostedRepo, token: Option<String>) -> Self {
        Self {
            fetch,
            repo_url: format!(
                "{API_URL}/repos/{}/{}",
                encode_path_segment(&remote.owner),
                encode_path_segment(&remote.repo)
            ),
            auth: Auth::token(token),
        }
    }

    pub(crate) fn repository(&self) -> Result<ForgejoRepo, ApiError> {
        self.get(&self.repo_url)
    }

    pub(crate) fn find_pull(
        &self,
        filter: PullFilter,
        matches: impl FnMut(&CodebergPrResponse) -> bool,
    ) -> PageScan {
        scan_pages(matches, |page| {
            self.get::<Vec<CodebergPrResponse>>(&format!(
                "{}/pulls?state={}&page={page}&limit={PAGE_SIZE}",
                self.repo_url,
                filter.query()
            ))
        })
    }

    pub(crate) fn create_pull(&self, pull: &CreatePull) -> Result<CodebergPrResponse, ApiError> {
        self.send(Method::Post, format!("{}/pulls", self.repo_url), pull)
    }

    pub(crate) fn edit_pull(&self, number: u32, pull: &EditPull) -> Result<(), ApiError> {
        let url = format!("{}/pulls/{number}", self.repo_url);
        self.send::<IgnoredAny>(Method::Patch, url, pull).map(drop)
    }

    pub(super) fn commit_status(&self, sha: &str) -> Result<CodebergCombinedStatus, ApiError> {
        self.get(&format!(
            "{}/commits/{}/status",
            self.repo_url,
            encode_path_segment(sha)
        ))
    }

    fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T, ApiError> {
        self.call(&Request::get(url, DEFAULT_TEXT_CAP, &self.auth))
    }

    fn send<T: DeserializeOwned>(
        &self,
        method: Method,
        url: String,
        body: &impl Serialize,
    ) -> Result<T, ApiError> {
        let request = Request::json(url, method, &self.auth, body)
            .expect("Codeberg request bodies are plain structs");
        self.call(&request)
    }

    fn call<T: DeserializeOwned>(&self, request: &Request) -> Result<T, ApiError> {
        let response = self
            .fetch
            .fetch(request)?
            .error_for_status()
            .map_err(|response| ApiError::from_response(&response))?;
        response.json().map_err(|_| ApiError::unexpected(&response))
    }
}
