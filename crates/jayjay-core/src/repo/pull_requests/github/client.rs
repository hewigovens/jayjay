use super::super::PrLookup;
use super::pull_request::parse_pr_json;
use crate::repo::Repo;
use crate::repo::environment::gh_binary;

fn pr_view_args(bookmark: &str) -> [&str; 6] {
    [
        "pr",
        "view",
        "--json",
        "number,state,title,url,statusCheckRollup",
        "--",
        bookmark,
    ]
}

pub(crate) fn pr_info(repo: &Repo, bookmark: &str) -> PrLookup {
    let Ok(output) = repo.command_output(&gh_binary(), &pr_view_args(bookmark), "gh pr view")
    else {
        return PrLookup::Unknown;
    };
    if !output.status.success() {
        // gh exits non-zero for both "no PR" and offline/auth errors; only the former is actionable.
        return if is_no_pr_error(&Repo::stderr_text(&output)) {
            PrLookup::NotFound
        } else {
            PrLookup::Unknown
        };
    }
    match parse_pr_json(&Repo::stdout_text(&output)) {
        Some(pr) => PrLookup::Found(pr),
        None => PrLookup::Unknown,
    }
}

/// `gh pr view` reports a confirmed absence with "no pull requests found".
fn is_no_pr_error(stderr: &str) -> bool {
    stderr.contains("no pull requests found") || stderr.contains("no open pull requests found")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pr_view_args_put_bookmark_after_separator() {
        assert_eq!(
            pr_view_args("feat-x"),
            [
                "pr",
                "view",
                "--json",
                "number,state,title,url,statusCheckRollup",
                "--",
                "feat-x"
            ]
        );
        // An option-shaped bookmark lands after `--`, never parsed as a flag.
        let args = pr_view_args("--repo=evil");
        assert_eq!(args[args.len() - 2], "--");
        assert_eq!(args[args.len() - 1], "--repo=evil");
    }
}
