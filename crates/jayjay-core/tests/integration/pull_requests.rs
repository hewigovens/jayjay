use std::path::PathBuf;

use jayjay_core::Repo;
use jj_test::{init_jj_repo, run_git, run_jj};

/// When the Codeberg PR listing can't be confirmed (404/offline/rate-limited), `pull_request_open_url` falls back to the host's new-PR (compose) URL. The compose page surfaces an existing PR for the branch instead of duplicating, so a working "Pull Request" action beats a dead one. Missing origin remotes are errors.
#[test]
fn codeberg_open_url_falls_back_to_compose_when_pr_status_unknown() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    run_git(
        &repo_path,
        &[
            "remote",
            "add",
            "origin",
            "https://codeberg.org/hewigovens/jayjay.git",
        ],
    );
    run_jj(&["-R", repo_str, "bookmark", "create", "master", "-r", "@"]);

    let repo = Repo::open(&repo_path).expect("open repo");

    assert_eq!(repo.pr_host_name(), Some("Codeberg".to_owned()));
    assert_eq!(
        repo.pull_request_open_url("feat/foo").unwrap(),
        "https://codeberg.org/hewigovens/jayjay/compare/master...feat/foo"
    );
}

/// Same for GitLab: an unconfirmed MR listing falls back to the new-MR compose URL rather than yielding a dead action.
#[test]
fn gitlab_open_url_falls_back_to_compose_when_mr_status_unknown() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    run_git(
        &repo_path,
        &[
            "remote",
            "add",
            "origin",
            "https://gitlab.com/hewigovens/jayjay.git",
        ],
    );
    run_jj(&["-R", repo_str, "bookmark", "create", "master", "-r", "@"]);

    let repo = Repo::open(&repo_path).expect("open repo");

    assert_eq!(repo.pr_host_name(), Some("GitLab".to_owned()));
    assert_eq!(
        repo.pull_request_open_url("feat/foo").unwrap(),
        "https://gitlab.com/hewigovens/jayjay/-/merge_requests/new?merge_request[source_branch]=feat/foo"
    );
}

/// Cursor Origin remotes open the codebase repo page when PR status cannot be confirmed. Origin has no compose URL, and JayJay will not create a PR until `origin pr list` confirms there is none.
#[test]
fn cursor_open_url_falls_back_to_repo_page_when_pr_status_unknown() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    run_git(
        &repo_path,
        &[
            "remote",
            "add",
            "origin",
            "https://origin.cursor.com/acme/checkout.git",
        ],
    );
    run_jj(&["-R", repo_str, "bookmark", "create", "main", "-r", "@"]);

    let repo = Repo::open(&repo_path).expect("open repo");

    assert_eq!(repo.pr_host_name(), Some("Cursor".to_owned()));
    assert_eq!(
        repo.pull_request_open_url("feat/foo").unwrap(),
        "https://cursor.com/codebase/acme/checkout"
    );
    assert_eq!(
        repo.remote_web_url().as_deref(),
        Some("https://cursor.com/codebase/acme/checkout")
    );
}

#[test]
fn open_url_errors_without_supported_origin() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    let error = repo.pull_request_open_url("feat/foo").unwrap_err();
    assert!(
        error
            .to_string()
            .contains("no GitHub, GitLab, Codeberg, or Cursor"),
        "{error}"
    );
}

/// Sibling checkout of `hewig/jayjay-origin-smoke` (standalone Origin repo, not a GitHub mirror). Skip when it is not next to the JayJay workspace.
fn sibling_origin_smoke() -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../jayjay-origin-smoke");
    path.canonicalize()
        .ok()
        .filter(|path| path.join(".jj").is_dir())
}

/// Live Origin fixture: existing PR opens, missing head is an error instead of the codebase page.
#[test]
fn cursor_origin_smoke_opens_existing_pr_and_errors_on_missing_head() {
    let Some(path) = sibling_origin_smoke() else {
        eprintln!("skipping: sibling jayjay-origin-smoke checkout not found");
        return;
    };
    let repo = Repo::open(&path).unwrap_or_else(|error| panic!("open {}: {error}", path.display()));
    assert_eq!(repo.pr_host_name().as_deref(), Some("Cursor"));

    let url = repo
        .pull_request_open_url("feat/origin-forge-smoke")
        .unwrap_or_else(|error| panic!("open existing Origin PR: {error}"));
    assert!(
        url.contains("/hewig/jayjay-origin-smoke/pull/"),
        "unexpected Origin PR URL: {url}"
    );

    let error = repo
        .pull_request_open_url("does-not-exist")
        .expect_err("missing Origin head must not open the codebase page");
    let message = error.to_string();
    assert!(
        message.to_ascii_lowercase().contains("does not exist"),
        "expected a missing-ref error, got: {message}"
    );
}
