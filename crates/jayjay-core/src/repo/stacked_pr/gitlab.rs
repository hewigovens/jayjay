use super::super::Repo;
use super::super::environment::glab_binary;
use super::forge::{ForgeTarget, failed, non_empty_or, number_from_url};
use crate::types::{StackLayerOutcome, SubmittedLayer};

/// Create an MR for `target` (source = its bookmark, target = the layer below), or
/// retarget an existing MR. Mirrors the GitHub path but via the `glab` CLI.
pub(super) fn create_or_update_mr(repo: &Repo, target: &ForgeTarget) -> SubmittedLayer {
    match existing_mr(repo, &target.bookmark) {
        Some((iid, url)) => update_target(repo, target, iid, url),
        None => create_mr(repo, target),
    }
}

fn existing_mr(repo: &Repo, head: &str) -> Option<(u32, String)> {
    let out = repo
        .command_output(
            &glab_binary(),
            &["mr", "view", head, "-F", "json"],
            "glab mr view",
        )
        .ok()?;
    if !out.status.success() {
        return None;
    }
    #[derive(serde::Deserialize)]
    struct View {
        iid: u32,
        web_url: String,
    }
    let view: View = serde_json::from_str(&Repo::stdout_text(&out)).ok()?;
    Some((view.iid, view.web_url))
}

fn update_target(repo: &Repo, target: &ForgeTarget, iid: u32, url: String) -> SubmittedLayer {
    // Refresh title/description too so editing a commit and re-running keeps the MR
    // in sync, not just its target branch.
    let title = non_empty_or(&target.title, &target.bookmark);
    let description = non_empty_or(&target.body, &title);
    let result = repo.command_output(
        &glab_binary(),
        &[
            "mr",
            "update",
            target.bookmark.as_str(),
            "--target-branch",
            target.base.as_str(),
            "--title",
            title.as_str(),
            "--description",
            description.as_str(),
            // Also enforce branch retention on re-submit so already-created MRs
            // become stack-safe.
            "--remove-source-branch=false",
        ],
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
        Ok(out) if out.status.success() => {
            let url = mr_url_from_text(&Repo::stdout_text(&out));
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

/// The MR web URL out of `glab mr create`'s output.
fn mr_url_from_text(text: &str) -> String {
    text.split_whitespace()
        .find(|token| token.contains("/merge_requests/"))
        .unwrap_or("")
        .trim()
        .to_owned()
}
