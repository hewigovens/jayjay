use jayjay_core::Repo;
use jj_test::{init_jj_repo, run_jj_in};

fn add_origin(repo_path: &std::path::Path, url: &str) {
    run_jj_in(repo_path, &["git", "remote", "add", "origin", url]);
}

#[test]
fn preview_requires_a_supported_origin_remote() {
    let temp = init_jj_repo();
    let repo_path = temp.path().join("repo");
    add_origin(&repo_path, "https://mirror.example.com/owner/base.git");
    let repo = Repo::open(&repo_path).expect("open repo");

    let error = repo
        .pull_request_import_preview(
            "https://github.com/hewigovens/jayjay/pull/290",
            &repo.sync_token(),
        )
        .expect_err("unsupported origin host must fail");
    assert!(
        error.to_string().contains("not on a supported host"),
        "{error}"
    );
}

#[test]
fn preview_rejects_pull_requests_for_a_different_base_repo() {
    let temp = init_jj_repo();
    let repo_path = temp.path().join("repo");
    add_origin(&repo_path, "https://github.com/owner/other.git");
    let repo = Repo::open(&repo_path).expect("open repo");

    let error = repo
        .pull_request_import_preview(
            "https://github.com/hewigovens/jayjay/pull/290",
            &repo.sync_token(),
        )
        .expect_err("mismatched base repo must fail");
    let message = error.to_string();
    assert!(message.contains("hewigovens/jayjay"), "{message}");
    assert!(message.contains("owner/other"), "{message}");
}
