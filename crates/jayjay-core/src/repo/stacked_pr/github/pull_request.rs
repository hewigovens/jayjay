use super::super::forge::{
    ForgeTarget, auth_preflight, created, edit_pull_request, failed, non_empty_or,
};
use crate::repo::{Repo, environment::gh_binary};
use crate::types::{CoreResult, SubmittedLayer};

pub(super) fn preflight(repo: &Repo) -> CoreResult<()> {
    auth_preflight(repo, &gh_binary(), "gh", "GitHub CLI (gh)")
}

pub(super) fn create_or_update_pr(repo: &Repo, target: &ForgeTarget) -> SubmittedLayer {
    match open_pr(repo, &target.bookmark) {
        Some((number, url)) => edit_pull_request(repo, &gh_binary(), "gh", target, number, url),
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
            created(target, title, Repo::stdout_text(&out).trim().to_owned())
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
