use jayjay_core::Repo;
use jj_test::{init_jj_repo, run_jj};

#[test]
fn detect_stack_builds_linear_layers_with_dependent_bases() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    // base → layer one → layer two(@)
    run_jj(&["-R", repo_str, "describe", "-m", "base change"]);
    run_jj(&["-R", repo_str, "bookmark", "create", "base", "-r", "@"]);
    run_jj(&["-R", repo_str, "new", "-m", "layer one"]);
    run_jj(&["-R", repo_str, "new", "-m", "layer two"]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let stack = repo.detect_stack("base", "@").expect("detect stack");

    assert_eq!(stack.layers.len(), 2);
    assert_eq!(stack.layers[0].title, "layer one");
    assert_eq!(stack.layers[1].title, "layer two");

    // Bottom PR targets the trunk branch; the upper PR targets the layer below.
    assert_eq!(stack.layers[0].base, stack.base_bookmark);
    assert_eq!(stack.layers[1].base, stack.layers[0].bookmark);

    // Bookmarks are auto-assigned (no existing ones) and slugged from the title.
    assert!(!stack.layers[0].bookmark_existed);
    assert!(
        stack.layers[0].bookmark.starts_with("layer-one-"),
        "got {}",
        stack.layers[0].bookmark
    );
}

#[test]
fn detect_stack_reuses_an_existing_bookmark() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    run_jj(&["-R", repo_str, "bookmark", "create", "base", "-r", "@"]);
    run_jj(&["-R", repo_str, "new", "-m", "feature"]);
    run_jj(&[
        "-R",
        repo_str,
        "bookmark",
        "create",
        "my-feature",
        "-r",
        "@",
    ]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let stack = repo.detect_stack("base", "@").expect("detect");

    assert_eq!(stack.layers.len(), 1);
    assert!(stack.layers[0].bookmark_existed);
    assert_eq!(stack.layers[0].bookmark, "my-feature");
}
