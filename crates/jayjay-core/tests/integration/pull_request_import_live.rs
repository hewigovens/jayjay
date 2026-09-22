//! Not run in CI: cargo test -p jayjay-core --test integration pull_request_import_live -- --ignored

use jayjay_core::Repo;
use jj_test::{init_jj_repo, run_jj_in};

#[test]
#[ignore = "live GitHub smoke test"]
fn live_github_fork_pr_preview_and_import() {
    let temp = init_jj_repo();
    let repo_path = temp.path().join("repo");
    run_jj_in(
        &repo_path,
        &[
            "git",
            "remote",
            "add",
            "origin",
            "https://github.com/hewigovens/jayjay.git",
        ],
    );
    let repo = Repo::open(&repo_path).expect("open repo");
    let url = "https://github.com/hewigovens/jayjay/pull/289";

    let preview = repo
        .pull_request_import_preview(url, &repo.sync_token())
        .expect("preview");
    assert_eq!(preview.pull_request.number, 289);
    assert_eq!(preview.pull_request.base_repo, "hewigovens/jayjay");
    assert!(!preview.same_repository);
    assert!(!preview.remote.exists);
    assert!(preview.existing_workspace.is_none());
    assert_eq!(preview.remote.name, "joshka");
    assert_eq!(preview.remote.bookmark, "joshka/macos-ux-polish");
    assert_eq!(preview.head_commit_id.len(), 40);
    assert!(preview.workspace.name.starts_with("pr-289"));

    let dest = temp.path().join(&preview.workspace.name);
    let created = repo
        .pull_request_import(
            url,
            &preview.head_commit_id,
            &preview.workspace.name,
            &dest.to_string_lossy(),
            &repo.sync_token(),
        )
        .expect("import");
    assert_eq!(created, dest.to_string_lossy());

    let parent = String::from_utf8(
        run_jj_in(&dest, &["log", "--no-graph", "-r", "@-", "-T", "commit_id"]).stdout,
    )
    .expect("utf8 log");
    assert_eq!(parent.trim(), preview.head_commit_id);

    let repo = Repo::open(&repo_path).expect("reopen repo");
    let again = repo
        .pull_request_import_preview(url, &repo.sync_token())
        .expect("preview again");
    assert!(again.remote.exists);
    assert_eq!(
        again.existing_workspace.map(|workspace| workspace.name),
        Some(preview.workspace.name)
    );
}
