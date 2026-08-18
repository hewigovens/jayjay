use jj_lib::revset::RevsetExpression;

use super::super::super::Repo;
use super::super::super::environment::origin_binary;
use super::super::PrLookup;
use super::mirror::{create_failure_message, pr_creation_info};
use crate::commit_message;

/// `origin pr create` argv; `--head` takes the bookmark so an option-shaped name is never a flag.
fn create_pr_args<'a>(
    head: &'a str,
    base: Option<&'a str>,
    title: &'a str,
    body: &'a str,
) -> Vec<&'a str> {
    let mut args = vec!["pr", "create", "--status", "open"];
    if let Some(base) = base {
        args.extend(["--base", base]);
    }
    args.extend(["--head", head, "--title", title, "--body", body]);
    args
}

pub(crate) fn create_pr(
    repo: &Repo,
    bookmark: &str,
    base: Option<&str>,
    title: &str,
    body: &str,
) -> Result<String, String> {
    let result = repo.command_output(
        &origin_binary(),
        &create_pr_args(bookmark, base, title, body),
        "origin pr create",
    );
    match result {
        Ok(out) if out.status.success() => {
            let combined = format!("{}\n{}", Repo::stdout_text(&out), Repo::stderr_text(&out));
            created_url(&combined)
                .ok_or_else(|| "origin pr create succeeded without a pull request URL".to_owned())
        }
        Ok(out) => {
            let combined = format!("{}\n{}", Repo::stdout_text(&out), Repo::stderr_text(&out));
            Err(create_failure_message(&combined))
        }
        Err(error) => Err(error.to_string()),
    }
}

/// Origin has no compose URL, so a confirmed miss creates via the CLI; lookup failures stay on the repo page. Create failures (GitHub inbound mirrors, missing remote bookmark) are errors, not a silent repo-page fallback.
pub(crate) fn open_or_create_url(
    repo: &Repo,
    bookmark: &str,
    lookup: PrLookup,
    fallback: String,
) -> Result<String, String> {
    open_or_create_url_with(lookup, fallback, || {
        pr_creation_info(repo)?;
        let (title, body) = title_and_body_for_bookmark(repo, bookmark);
        create_pr(repo, bookmark, None, &title, &body)
    })
}

fn open_or_create_url_with(
    lookup: PrLookup,
    fallback: String,
    create: impl FnOnce() -> Result<String, String>,
) -> Result<String, String> {
    match lookup {
        PrLookup::Found(pr) => Ok(pr.url),
        PrLookup::NotFound => create(),
        PrLookup::Unknown => Ok(fallback),
    }
}

fn title_and_body_for_bookmark(repo: &Repo, bookmark: &str) -> (String, String) {
    let description = repo
        .log_typed(RevsetExpression::symbol(bookmark.to_owned()))
        .ok()
        .and_then(|changes| changes.into_iter().next())
        .map(|change| change.description)
        .unwrap_or_default();
    let title = commit_message::summary(&description);
    let body = commit_message::body(&description);
    (
        if title.is_empty() {
            bookmark.to_owned()
        } else {
            title
        },
        body,
    )
}

fn created_url(text: &str) -> Option<String> {
    text.split_whitespace()
        .find(|token| token.contains("/pull/"))
        .map(|token| token.trim().to_owned())
        .filter(|url| !url.is_empty())
}

#[cfg(test)]
mod tests {
    use super::super::super::PrLookup;
    use super::super::mirror::GITHUB_MIRROR_BLOCK;
    use super::{
        create_pr_args, created_url, open_or_create_url_with, title_and_body_for_bookmark,
    };
    use crate::Repo;
    use crate::types::{ChecksStatus, PrInfo, PrState};
    use jj_test::{init_jj_repo, run_jj};

    fn pr() -> PrInfo {
        PrInfo {
            number: 1,
            state: PrState::Open,
            title: "t".into(),
            url: "https://cursor.com/codebase/o/r/pull/1".into(),
            checks: ChecksStatus::None,
        }
    }

    #[test]
    fn create_only_on_confirmed_absence() {
        assert_eq!(
            open_or_create_url_with(PrLookup::Found(pr()), "FALLBACK".into(), || panic!(
                "must not create when a PR already exists"
            ))
            .unwrap(),
            "https://cursor.com/codebase/o/r/pull/1"
        );
        assert_eq!(
            open_or_create_url_with(PrLookup::NotFound, "FALLBACK".into(), || Ok(
                "https://cursor.com/codebase/o/r/pull/9".into()
            ))
            .unwrap(),
            "https://cursor.com/codebase/o/r/pull/9"
        );
        assert_eq!(
            open_or_create_url_with(PrLookup::NotFound, "FALLBACK".into(), || {
                Err(GITHUB_MIRROR_BLOCK.to_owned())
            })
            .unwrap_err(),
            GITHUB_MIRROR_BLOCK
        );
        assert_eq!(
            open_or_create_url_with(PrLookup::Unknown, "FALLBACK".into(), || panic!(
                "must not create when PR status is unconfirmed"
            ))
            .unwrap(),
            "FALLBACK"
        );
    }

    #[test]
    fn argv_uses_forge_default_for_direct_create_and_explicit_stack_base() {
        let args = create_pr_args("--repo=evil", None, "title", "body");
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--head", "--repo=evil"])
        );
        assert!(args.windows(2).any(|pair| pair == ["--status", "open"]));
        assert!(!args.contains(&"--base"));

        let stacked = create_pr_args("feature", Some("develop"), "title", "body");
        assert!(stacked.windows(2).any(|pair| pair == ["--base", "develop"]));
    }

    #[test]
    fn title_and_body_resolve_bookmark_as_a_literal_symbol() {
        let temp_dir = init_jj_repo();
        let repo_path = temp_dir.path().join("repo");
        let repo_str = repo_path.to_str().expect("repo path utf-8");
        run_jj(&[
            "-R",
            repo_str,
            "describe",
            "-m",
            "literal title\n\nliteral body",
        ]);
        run_jj(&["-R", repo_str, "bookmark", "create", "\"all()\"", "-r", "@"]);
        run_jj(&[
            "-R",
            repo_str,
            "new",
            "-m",
            "unrelated title\n\nunrelated body",
        ]);

        let repo = Repo::open(&repo_path).expect("open repo");
        assert_eq!(
            title_and_body_for_bookmark(&repo, "all()"),
            ("literal title".to_owned(), "literal body".to_owned())
        );
    }

    #[test]
    fn create_output_url_comes_from_the_url_line() {
        let text = "Created pull request #12 in acme/checkout.\n  URL:         https://cursor.com/codebase/acme/checkout/pull/12\n";
        assert_eq!(
            created_url(text).as_deref(),
            Some("https://cursor.com/codebase/acme/checkout/pull/12")
        );
        assert_eq!(created_url("no url here"), None);
    }
}
