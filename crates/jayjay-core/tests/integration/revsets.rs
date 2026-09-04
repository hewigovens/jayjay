use std::fs;

use jayjay_core::{DEFAULT_REVSET, Repo, combined_diff_revsets, revset_presets};
use jj_test::{init_jj_repo, run_jj};

#[test]
fn combined_diff_spans_roots_to_heads_of_selection() {
    let revisions = vec![
        "newest".to_owned(),
        "middle".to_owned(),
        "oldest".to_owned(),
    ];

    let (from, to) =
        combined_diff_revsets(&revisions).expect("three revisions form a diff selection");

    assert_eq!(from, "roots((newest) | (middle) | (oldest))-");
    assert_eq!(to, "heads((newest) | (middle) | (oldest))");
}

#[test]
fn combined_diff_requires_two_unique_revisions() {
    assert!(combined_diff_revsets(&["only".to_owned()]).is_none());
    assert!(combined_diff_revsets(&["same".to_owned(), "same".to_owned()]).is_none());
}

#[test]
fn combined_diff_matches_oldest_parent_to_newest() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    fs::write(repo_path.join("stack.txt"), "oldest\n").expect("write oldest");
    run_jj(&["-R", repo_str, "describe", "-m", "oldest"]);
    run_jj(&["-R", repo_str, "new", "-m", "middle"]);
    fs::write(repo_path.join("stack.txt"), "middle\n").expect("write middle");
    run_jj(&["-R", repo_str, "new", "-m", "newest"]);
    fs::write(repo_path.join("stack.txt"), "newest\n").expect("write newest");
    run_jj(&["-R", repo_str, "st"]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let log = repo.log("all()").expect("load stack");
    let commit_id = |description: &str| {
        log.iter()
            .find(|change| change.description.trim() == description)
            .unwrap_or_else(|| panic!("missing {description}"))
            .commit_id
            .id
            .clone()
    };
    let newest = commit_id("newest");
    let middle = commit_id("middle");
    let oldest = commit_id("oldest");
    let (from, to) = combined_diff_revsets(&[newest.clone(), middle, oldest.clone()])
        .expect("build combined diff revsets");

    let combined = repo
        .interdiff_file(&from, &to, "stack.txt")
        .expect("load combined diff");
    let direct = repo
        .interdiff_file(&format!("{oldest}-"), &newest, "stack.txt")
        .expect("load endpoint diff");

    assert_eq!(combined.old.content, direct.old.content);
    assert_eq!(combined.new.content, direct.new.content);
}

#[test]
fn default_revset_shows_nearby_heads() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    run_jj(&["-R", repo_str, "new", "@", "-m", "current head"]);
    run_jj(&["-R", repo_str, "new", "@-", "-m", "parallel head"]);

    let repo = Repo::open(&repo_path).expect("open repo");

    let log = repo.log(DEFAULT_REVSET).expect("evaluate default revset");
    assert!(
        !log.is_empty(),
        "default revset should evaluate to visible changes"
    );
    assert!(
        log.iter()
            .any(|change| change.description.trim_end() == "parallel head"),
        "expected default revset to include the current head"
    );
    assert!(
        log.iter()
            .any(|change| change.description.trim_end() == "current head"),
        "expected default revset to include nearby sibling heads"
    );
    assert!(
        log.iter()
            .any(|change| change.description.trim_end() == "initial change"),
        "expected default revset to keep trunk/root context visible"
    );
}
#[test]
fn trunk_revset_alias_is_available_in_app_parser() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let log = repo.log("trunk() | @").expect("evaluate trunk() revset");
    assert!(
        log.iter()
            .any(|change| change.description.trim_end() == "initial change"),
        "expected trunk() expression to parse and include current visible work"
    );
}
#[test]
fn immutable_heads_revset_alias_is_available_in_app_parser() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    let log = repo
        .log("present(@) | ancestors(immutable_heads().., 20) | trunk()")
        .expect("evaluate immutable_heads() revset");
    assert!(
        log.iter().any(|change| change.is_working_copy),
        "expected immutable_heads() expression to include the working copy"
    );
    assert!(
        log.iter()
            .any(|change| change.description.trim_end() == "initial change"),
        "expected immutable_heads() expression to parse alongside trunk()"
    );
}

#[test]
fn default_revset_evaluates_in_cli_and_app_parser() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    let cli = run_jj(&[
        "-R",
        repo_str,
        "log",
        "--no-graph",
        "-r",
        DEFAULT_REVSET,
        "-T",
        "commit_id.short() ++ \"\\n\"",
    ]);
    assert!(
        !cli.stdout.is_empty(),
        "jj CLI should evaluate JayJay's default revset"
    );

    let repo = Repo::open(&repo_path).expect("open repo");
    let app = repo.log(DEFAULT_REVSET).expect("evaluate default revset");
    assert!(
        !app.is_empty(),
        "JayJay should evaluate the same default revset as the jj CLI"
    );
}

#[test]
fn custom_immutable_heads_alias_can_reference_builtin_default_alias() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    run_jj(&[
        "-R",
        repo_str,
        "config",
        "set",
        "--repo",
        r#"revset-aliases."immutable_heads()""#,
        "builtin_immutable_heads() | root()",
    ]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let log = repo
        .log(DEFAULT_REVSET)
        .expect("evaluate user immutable_heads() alias");
    assert!(
        log.iter().any(|change| change.is_working_copy),
        "expected immutable_heads() alias to parse through builtin_immutable_heads()"
    );
}
#[test]
fn filter_presets_evaluate_in_app_parser() {
    let temp_dir = init_jj_repo();
    let repo = Repo::open(&temp_dir.path().join("repo")).expect("open repo");

    for preset in revset_presets() {
        repo.log(&preset.revset)
            .unwrap_or_else(|error| panic!("{} preset failed: {error}", preset.id));
    }
}
