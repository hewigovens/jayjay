use jayjay_core::Repo;
use jj_test::{init_jj_repo, run_git, run_jj};

/// When the Codeberg PR listing can't be confirmed (404/offline/rate-limited), `pull_request_open_url` falls back to the host's new-PR (compose) URL. The compose page surfaces an existing PR for the branch instead of duplicating, so a working "Pull Request" action beats a dead one. `None` is reserved for "no supported origin remote".
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
        repo.pull_request_open_url("feat/foo").as_deref(),
        Some("https://codeberg.org/hewigovens/jayjay/compare/master...feat/foo")
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
        repo.pull_request_open_url("feat/foo").as_deref(),
        Some(
            "https://gitlab.com/hewigovens/jayjay/-/merge_requests/new?merge_request[source_branch]=feat/foo"
        )
    );
}
