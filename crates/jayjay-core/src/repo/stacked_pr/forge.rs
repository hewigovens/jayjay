use crate::repo::Repo;
use crate::types::{CoreError, CoreResult, StackLayerOutcome, SubmittedLayer};

/// One PR/MR target — the subset of a stack layer the forge clients use.
pub(super) struct ForgeTarget {
    pub bookmark: String,
    pub base: String,
    pub title: String,
    pub body: String,
}

/// `value` if non-empty, else `fallback` — for title/description fallbacks.
pub(super) fn non_empty_or(value: &str, fallback: &str) -> String {
    if value.is_empty() {
        fallback.to_owned()
    } else {
        value.to_owned()
    }
}

/// The numeric PR/MR id from a `.../<n>` URL.
fn number_from_url(url: &str) -> Option<u32> {
    url.trim_end_matches('/').rsplit('/').next()?.parse().ok()
}

pub(super) fn failed(target: &ForgeTarget, detail: String) -> SubmittedLayer {
    SubmittedLayer {
        bookmark: target.bookmark.clone(),
        base: target.base.clone(),
        title: target.title.clone(),
        outcome: StackLayerOutcome::Failed,
        pr_number: 0,
        pr_url: String::new(),
        detail: detail.trim().to_owned(),
    }
}

pub(super) fn created(target: &ForgeTarget, title: String, url: String) -> SubmittedLayer {
    SubmittedLayer {
        bookmark: target.bookmark.clone(),
        base: target.base.clone(),
        title,
        outcome: StackLayerOutcome::Created,
        pr_number: number_from_url(&url).unwrap_or(0),
        pr_url: url,
        detail: format!("Created onto {}", target.base),
    }
}

/// First whitespace token containing `needle`, used to scrape a PR/MR URL from CLI output.
pub(super) fn url_containing(text: &str, needle: &str) -> String {
    text.split_whitespace()
        .find(|token| token.contains(needle))
        .unwrap_or("")
        .trim()
        .to_owned()
}

/// Prove `cli auth status` succeeds before any bookmark move or push.
pub(super) fn auth_preflight(
    repo: &Repo,
    binary: &str,
    cli: &str,
    display: &str,
) -> CoreResult<()> {
    let label = format!("{cli} auth status");
    match repo.command_output(binary, &["auth", "status"], &label) {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(CoreError::Internal {
            message: format!(
                "{display} is not ready: {}. Run `{cli} auth login` and retry.",
                Repo::stderr_text(&out).trim()
            ),
        }),
        Err(error) => Err(CoreError::Internal {
            message: format!(
                "{display} is unavailable: {error}. Install {cli} and run `{cli} auth login`."
            ),
        }),
    }
}

/// Shared `pr edit --base/--title/--body` used by GitHub (`gh`) and Cursor Origin (`origin`).
pub(super) fn edit_pull_request(
    repo: &Repo,
    binary: &str,
    cli: &str,
    target: &ForgeTarget,
    number: u32,
    url: String,
) -> SubmittedLayer {
    let num = number.to_string();
    let title = non_empty_or(&target.title, &target.bookmark);
    let label = format!("{cli} pr edit");
    let result = repo.command_output(
        binary,
        &[
            "pr",
            "edit",
            num.as_str(),
            "--base",
            target.base.as_str(),
            "--title",
            title.as_str(),
            "--body",
            target.body.as_str(),
        ],
        &label,
    );
    let (outcome, detail) = match result {
        Ok(out) if out.status.success() => (
            StackLayerOutcome::Updated,
            format!("PR #{number} → base {}", target.base),
        ),
        Ok(out) => (
            StackLayerOutcome::Failed,
            Repo::stderr_text(&out).trim().to_owned(),
        ),
        Err(error) => (StackLayerOutcome::Failed, error.to_string()),
    };
    SubmittedLayer {
        bookmark: target.bookmark.clone(),
        base: target.base.clone(),
        title: target.title.clone(),
        outcome,
        pr_number: number,
        pr_url: url,
        detail,
    }
}
