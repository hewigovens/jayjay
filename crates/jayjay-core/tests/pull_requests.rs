use jayjay_core::Repo;
use jj_test::{init_jj_repo, run_git, run_jj};

/// When the Codeberg PR listing can't be confirmed (404/offline/rate-limited),
/// `pull_request_open_url` must return None, never a compose URL that could
/// open a duplicate create-PR page for a bookmark that already has an open PR.
#[test]
fn codeberg_open_url_is_none_when_pr_status_unknown() {
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
    assert_eq!(repo.pull_request_open_url("feat/foo"), None);
}

/// Same guarantee for GitLab: an unconfirmed MR listing (404/offline/rate-limited)
/// must yield None rather than a compose URL that could duplicate an existing MR.
#[test]
fn gitlab_open_url_is_none_when_mr_status_unknown() {
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
    assert_eq!(repo.pull_request_open_url("feat/foo"), None);
}
