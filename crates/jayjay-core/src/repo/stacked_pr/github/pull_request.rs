use super::super::forge::{ForgeTarget, failed, non_empty_or, number_from_url};
use crate::repo::{Repo, environment::gh_binary};
use crate::types::{CoreError, CoreResult, StackLayerOutcome, SubmittedLayer};

pub(super) fn preflight(repo: &Repo) -> CoreResult<()> {
    match repo.command_output(&gh_binary(), &["auth", "status"], "gh auth status") {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => Err(CoreError::Internal {
            message: format!(
                "GitHub CLI (gh) is not ready: {}. Run `gh auth login` and retry.",
                Repo::stderr_text(&out).trim()
            ),
        }),
        Err(error) => Err(CoreError::Internal {
            message: format!(
                "GitHub CLI (gh) is unavailable: {error}. Install gh and run `gh auth login`."
            ),
        }),
    }
}

pub(super) fn create_or_update_pr(repo: &Repo, target: &ForgeTarget) -> SubmittedLayer {
    match open_pr(repo, &target.bookmark) {
        Some((number, url)) => update_base(repo, target, number, url),
        None => create_pr(repo, target),
    }
}

fn open_pr_args(head: &str) -> [&str; 10] {
    [
        "pr",
        "list",
        "--state",
        "open",
        "--limit",
        "1",
        "--json",
        "number,url",
        "--head",
        head,
    ]
}

fn open_pr(repo: &Repo, head: &str) -> Option<(u32, String)> {
    let out = repo
        .command_output(&gh_binary(), &open_pr_args(head), "gh pr list")
        .ok()?;
    if !out.status.success() {
        return None;
    }
    #[derive(serde::Deserialize)]
    struct View {
        number: u32,
        url: String,
    }
    let views: Vec<View> = serde_json::from_str(&Repo::stdout_text(&out)).ok()?;
    views.into_iter().next().map(|view| (view.number, view.url))
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

#[cfg(test)]
mod tests {
    use super::open_pr_args;

    #[test]
    fn open_pr_lookup_is_scoped_to_open_state_and_head_option() {
        let args = open_pr_args("feat-x");
        assert!(args.windows(2).any(|pair| pair == ["--state", "open"]));
        assert_eq!(args[args.len() - 2..], ["--head", "feat-x"]);
        // An option-shaped bookmark is consumed as --head's value, never parsed as a flag.
        let args = open_pr_args("--repo=evil");
        assert_eq!(args[args.len() - 2..], ["--head", "--repo=evil"]);
    }
}
