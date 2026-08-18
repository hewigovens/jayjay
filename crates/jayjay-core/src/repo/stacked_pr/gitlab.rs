use super::super::Repo;
use super::super::environment::glab_binary;
use super::forge::{ForgeTarget, auth_preflight, created, failed, non_empty_or, url_containing};
use crate::types::{CoreResult, StackLayerOutcome, SubmittedLayer};

/// Prove `glab` is installed and authenticated before any bookmark move or push,
/// so a missing/misconfigured CLI fails up front instead of leaving dangling
/// remote branches and moved local bookmarks with no MRs.
pub(super) fn preflight(repo: &Repo) -> CoreResult<()> {
    auth_preflight(repo, &glab_binary(), "glab", "GitLab CLI (glab)")
}

/// Create an MR for `target` (source = its bookmark, target = the layer below), or
/// retarget an existing MR. Mirrors the GitHub path but via the `glab` CLI.
pub(super) fn create_or_update_mr(repo: &Repo, target: &ForgeTarget) -> SubmittedLayer {
    match existing_mr(repo, &target.bookmark) {
        Some((iid, url)) => update_target(repo, target, iid, url),
        None => create_mr(repo, target),
    }
}

/// `glab mr view` argv with `head` after `--`, so an option-shaped bookmark name
/// can't be parsed as a flag.
fn mr_view_args(head: &str) -> [&str; 6] {
    ["mr", "view", "-F", "json", "--", head]
}

/// `glab mr update` argv with the bookmark after `--` and all options before it.
fn mr_update_args<'a>(
    base: &'a str,
    title: &'a str,
    description: &'a str,
    bookmark: &'a str,
) -> [&'a str; 11] {
    [
        "mr",
        "update",
        "--target-branch",
        base,
        "--title",
        title,
        "--description",
        description,
        // Enforce branch retention on re-submit so already-created MRs stay stack-safe.
        "--remove-source-branch=false",
        "--",
        bookmark,
    ]
}

fn existing_mr(repo: &Repo, head: &str) -> Option<(u32, String)> {
    let out = repo
        .command_output(&glab_binary(), &mr_view_args(head), "glab mr view")
        .ok()?;
    if !out.status.success() {
        return None;
    }
    #[derive(serde::Deserialize)]
    struct View {
        iid: u32,
        web_url: String,
        state: String,
    }
    let view: View = serde_json::from_str(&Repo::stdout_text(&out)).ok()?;
    // A closed or merged MR on the same branch must not be reused (its target can't be edited); create a fresh MR instead.
    (view.state == "opened").then_some((view.iid, view.web_url))
}

fn update_target(repo: &Repo, target: &ForgeTarget, iid: u32, url: String) -> SubmittedLayer {
    // Refresh title/description too so editing a commit and re-running keeps the MR
    // in sync, not just its target branch.
    let title = non_empty_or(&target.title, &target.bookmark);
    let description = non_empty_or(&target.body, &title);
    let result = repo.command_output(
        &glab_binary(),
        &mr_update_args(
            target.base.as_str(),
            title.as_str(),
            description.as_str(),
            target.bookmark.as_str(),
        ),
        "glab mr update",
    );
    let (outcome, detail) = match result {
        Ok(out) if out.status.success() => (
            StackLayerOutcome::Updated,
            format!("MR !{iid} → base {}", target.base),
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
        pr_number: iid,
        pr_url: url,
        detail,
    }
}

fn create_mr(repo: &Repo, target: &ForgeTarget) -> SubmittedLayer {
    let title = non_empty_or(&target.title, &target.bookmark);
    let description = non_empty_or(&target.body, &title);
    let result = repo.command_output(
        &glab_binary(),
        &[
            "mr",
            "create",
            "--source-branch",
            target.bookmark.as_str(),
            "--target-branch",
            target.base.as_str(),
            "--title",
            title.as_str(),
            "--description",
            description.as_str(),
            // Keep the source branch on merge: GitLab only auto-retargets the MR
            // above to `main` when the merged branch survives. Deleting it would
            // orphan the rest of the stack. Branches are cleaned up after landing.
            "--remove-source-branch=false",
            "--yes",
        ],
        "glab mr create",
    );
    match result {
        Ok(out) if out.status.success() => created(
            target,
            title,
            url_containing(&Repo::stdout_text(&out), "/merge_requests/"),
        ),
        Ok(out) => failed(target, Repo::stderr_text(&out)),
        Err(error) => failed(target, error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::{mr_update_args, mr_view_args};

    #[test]
    fn mr_view_args_put_head_after_separator() {
        assert_eq!(
            mr_view_args("feat-x"),
            ["mr", "view", "-F", "json", "--", "feat-x"]
        );
        let args = mr_view_args("--repo=evil");
        assert_eq!(args[args.len() - 2], "--");
        assert_eq!(args[args.len() - 1], "--repo=evil");
    }

    #[test]
    fn mr_update_args_put_bookmark_after_separator() {
        assert_eq!(
            mr_update_args("main", "T", "D", "feat-x"),
            [
                "mr",
                "update",
                "--target-branch",
                "main",
                "--title",
                "T",
                "--description",
                "D",
                "--remove-source-branch=false",
                "--",
                "feat-x",
            ]
        );
        // An option-shaped bookmark lands after `--`, never parsed as a flag.
        let evil = mr_update_args("main", "T", "D", "--yes");
        assert_eq!(evil[evil.len() - 2], "--");
        assert_eq!(evil[evil.len() - 1], "--yes");
    }
}
