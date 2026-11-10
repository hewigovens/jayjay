use std::path::PathBuf;

use jayjay_core::Repo;
use jj_test::init_jj_repo;

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
