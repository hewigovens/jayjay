use super::super::Repo;
use super::super::environment::origin_binary;
use super::forge::{ForgeTarget, auth_preflight, created, edit_pull_request, failed, non_empty_or};
use crate::repo::pull_requests::cursor;
use crate::types::{CoreError, CoreResult, SubmittedLayer};

pub(super) fn default_branch(repo: &Repo) -> CoreResult<String> {
    cursor::pr_creation_info(repo)
        .and_then(|info| info.default_branch().map(str::to_owned))
        .map_err(CoreError::internal)
}

pub(super) fn preflight(repo: &Repo) -> CoreResult<String> {
    auth_preflight(repo, &origin_binary(), "origin", "Cursor Origin CLI")?;
    default_branch(repo)
}

pub(super) fn create_or_update_pr(repo: &Repo, target: &ForgeTarget) -> SubmittedLayer {
    match cursor::open_pr(repo, &target.bookmark) {
        Ok(Some((number, url))) => {
            edit_pull_request(repo, &origin_binary(), "origin", target, number, url)
        }
        Ok(None) => create_pr(repo, target),
        Err(detail) => failed(target, detail),
    }
}

fn create_pr(repo: &Repo, target: &ForgeTarget) -> SubmittedLayer {
    let title = non_empty_or(&target.title, &target.bookmark);
    match cursor::create_pr(
        repo,
        &target.bookmark,
        Some(&target.base),
        &title,
        &target.body,
    ) {
        Ok(url) => created(target, title, url),
        Err(detail) => failed(target, detail),
    }
}
