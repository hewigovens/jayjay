use jayjay_core::Repo;
use jj_test::{init_jj_repo, run_git, run_jj};

#[test]
fn codeberg_pull_request_url_uses_origin_and_default_base_bookmark() {
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

    assert_eq!(repo.pull_request_host_name(), Some("Codeberg".to_owned()));
    assert_eq!(
        repo.pull_request_open_url("feat/foo"),
        Some("https://codeberg.org/hewigovens/jayjay/compare/master...feat/foo".to_owned())
    );
}
