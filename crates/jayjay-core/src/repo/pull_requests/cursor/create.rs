use jj_lib::revset::RevsetExpression;

use super::super::super::Repo;
use super::super::PrLookup;
use super::api::Api;
use crate::commit_message;
use crate::repo::hosted_repo::HostedRepo;

/// Origin has no compose URL, so a confirmed miss creates through the API; lookup failures stay on the repo page. Create failures (GitHub inbound mirrors, missing remote bookmark) are errors, not a silent repo-page fallback.
pub(crate) fn open_or_create_url(
    repo: &Repo,
    remote: &HostedRepo,
    bookmark: &str,
    lookup: PrLookup,
) -> Result<String, String> {
    open_or_create_url_with(lookup, remote.web_url(), || {
        let api = Api::for_remote(remote);
        let base = api.pull_request_base()?;
        let (title, body) = title_and_body_for_bookmark(repo, bookmark);
        api.create_pull(bookmark, &base, &title, &body)
            .map(|pull| pull.web_url)
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

#[cfg(test)]
mod tests {
    use super::super::super::PrLookup;
    use super::super::repo::GITHUB_MIRROR_BLOCK;
    use super::{open_or_create_url_with, title_and_body_for_bookmark};
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
}
