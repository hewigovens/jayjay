use super::super::Repo;
use super::super::environment::origin_binary;
use super::forge::{ForgeTarget, created, failed, non_empty_or, updated};
use crate::repo::pull_requests::cursor;
use crate::types::{JayError, JayResult, SubmittedLayer};

pub(super) fn default_branch(repo: &Repo) -> JayResult<String> {
    cursor::pr_creation_info(repo)
        .and_then(|info| info.default_branch().map(str::to_owned))
        .map_err(JayError::internal)
}

pub(super) fn preflight(repo: &Repo) -> JayResult<String> {
    match repo.command_output(&origin_binary(), &["auth", "status"], "origin auth status") {
        Ok(out) if out.status.success() => {}
        Ok(out) => {
            return Err(JayError::internal(format!(
                "Cursor Origin CLI is not ready: {}. Run `origin auth login` and retry.",
                Repo::stderr_text(&out).trim()
            )));
        }
        Err(error) => {
            return Err(JayError::internal(format!(
                "Cursor Origin CLI is unavailable: {error}. Install origin and run `origin auth login`."
            )));
        }
    }
    default_branch(repo)
}

pub(super) fn create_or_update_pr(repo: &Repo, target: &ForgeTarget) -> SubmittedLayer {
    match cursor::open_pr(repo, &target.bookmark) {
        Ok(Some((number, url))) => edit_pr(repo, target, number, url),
        Ok(None) => create_pr(repo, target),
        Err(detail) => failed(target, detail),
    }
}

fn edit_pr(repo: &Repo, target: &ForgeTarget, number: u32, url: String) -> SubmittedLayer {
    let num = number.to_string();
    let title = non_empty_or(&target.title, &target.bookmark);
    let args = [
        "pr",
        "edit",
        num.as_str(),
        "--base",
        target.base.as_str(),
        "--title",
        title.as_str(),
        "--body",
        target.body.as_str(),
    ];
    match repo.command_output(&origin_binary(), &args, "origin pr edit") {
        Ok(out) if out.status.success() => updated(
            target,
            target.title.clone(),
            number,
            url,
            format!("PR #{number}"),
        ),
        Ok(out) => failed(target, Repo::stderr_text(&out)),
        Err(error) => failed(target, error.to_string()),
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
