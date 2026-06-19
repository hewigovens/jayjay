use super::super::Repo;
use super::super::environment::gh_binary;
use super::forge::{ForgeTarget, failed, non_empty_or, number_from_url};
use crate::types::{StackLayerOutcome, SubmittedLayer};

/// Create a PR for `target` (head = its bookmark, base = the layer below), or
/// retarget an existing PR's base. Best-effort: each layer's failure is captured.
pub(super) fn create_or_update_pr(repo: &Repo, target: &ForgeTarget) -> SubmittedLayer {
    match existing_pr(repo, &target.bookmark) {
        Some((number, url)) => update_base(repo, target, number, url),
        None => create_pr(repo, target),
    }
}

fn existing_pr(repo: &Repo, head: &str) -> Option<(u32, String)> {
    let out = repo
        .command_output(
            &gh_binary(),
            &["pr", "view", head, "--json", "number,url"],
            "gh pr view",
        )
        .ok()?;
    if !out.status.success() {
        return None;
    }
    #[derive(serde::Deserialize)]
    struct View {
        number: u32,
        url: String,
    }
    let view: View = serde_json::from_str(&Repo::stdout_text(&out)).ok()?;
    Some((view.number, view.url))
}

fn update_base(repo: &Repo, target: &ForgeTarget, number: u32, url: String) -> SubmittedLayer {
    let num = number.to_string();
    // Refresh title/body too, so editing a commit description and re-running keeps
    // the PR in sync (not just the dependent base).
    let title = non_empty_or(&target.title, &target.bookmark);
    let result = repo.command_output(
        &gh_binary(),
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
        "gh pr edit",
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

fn create_pr(repo: &Repo, target: &ForgeTarget) -> SubmittedLayer {
    let title = non_empty_or(&target.title, &target.bookmark);
    let result = repo.command_output(
        &gh_binary(),
        &[
            "pr",
            "create",
            "--base",
            target.base.as_str(),
            "--head",
            target.bookmark.as_str(),
            "--title",
            title.as_str(),
            "--body",
            target.body.as_str(),
        ],
        "gh pr create",
    );
    match result {
        Ok(out) if out.status.success() => {
            let url = Repo::stdout_text(&out).trim().to_owned();
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
        Ok(out) => failed(target, Repo::stderr_text(&out)),
        Err(error) => failed(target, error.to_string()),
    }
}
