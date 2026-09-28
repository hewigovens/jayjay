use std::fs;

use jayjay_core::compare::combined_diff_revsets;
use jayjay_core::{DEFAULT_REVSET, Repo, bookmark_filter_revset};
use jj_test::{init_jj_repo, run_jj};

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
fn bookmark_filter_shows_the_stack_and_check_rejects_what_the_graph_cannot_load() {
    let temp = init_jj_repo();
    let path = temp.path().join("repo");
    let repo_str = path.to_str().unwrap();
    run_jj(&["-R", repo_str, "describe", "-m", "one"]);
    run_jj(&["-R", repo_str, "new", "-m", "two"]);
    run_jj(&["-R", repo_str, "bookmark", "create", "feature"]);
    run_jj(&["-R", repo_str, "new", "-m", "after"]);
    let repo = Repo::open(&path).unwrap();

    let revset = bookmark_filter_revset("feature", None);
    let changes = repo.log(&revset).unwrap();
    let descriptions: Vec<_> = changes.iter().map(|c| c.description.trim()).collect();
    assert_eq!(descriptions, ["two", "one"]);

    repo.check_revset(&revset).unwrap();
    for invalid in ["mine() & ::@)", "no-such-bookmark"] {
        assert!(repo.check_revset(invalid).is_err(), "{invalid}");
    }
}

#[test]
fn ancestors_filter_includes_merge_parents_but_excludes_other_heads() {
    let temp = init_jj_repo();
    let path = temp.path().join("repo");
    let repo_str = path.to_str().unwrap();
    run_jj(&["-R", repo_str, "describe", "-m", "base"]);
    run_jj(&["-R", repo_str, "bookmark", "create", "base"]);
    run_jj(&["-R", repo_str, "new", "base", "-m", "left"]);
    run_jj(&["-R", repo_str, "bookmark", "create", "left"]);
    run_jj(&["-R", repo_str, "new", "base", "-m", "right"]);
    run_jj(&["-R", repo_str, "new", "left", "@", "-m", "merge"]);
    let repo = Repo::open(&path).unwrap();
    let target = repo.log("@").unwrap().remove(0).change_id.id;
    let revset = jayjay_core::ancestors_revset(&target);
    run_jj(&["-R", repo_str, "describe", "-m", "merge rewritten"]);
    run_jj(&["-R", repo_str, "new", "base", "-m", "unrelated"]);
    let repo = Repo::open(&path).unwrap();
    let changes = repo.log(&revset).unwrap();
    let mut descriptions: Vec<_> = changes.iter().map(|c| c.description.trim()).collect();
    descriptions.sort_unstable();
    assert_eq!(descriptions, ["base", "left", "merge rewritten", "right"]);
}

#[test]
fn revset_vocabulary_offers_the_refs_tags_and_aliases_of_the_repository() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    run_jj(&["-R", repo_str, "bookmark", "create", "feature"]);
    // jj names a bookmark it cannot read as a symbol by its quoted symbol, which is what a completion inserts.
    run_jj(&["-R", repo_str, "bookmark", "create", r#""fix-a|b""#]);
    run_jj(&["-R", repo_str, "tag", "set", "v1.0.0", "-r", "@"]);
    run_jj(&[
        "-R",
        repo_str,
        "config",
        "set",
        "--repo",
        "revset-aliases.wip",
        "description(wip)",
    ]);
    run_jj(&[
        "-R",
        repo_str,
        "config",
        "set",
        "--repo",
        r#"revset-aliases."reviewed()""#,
        "@",
    ]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let vocabulary = repo.revset_vocabulary(&repo.list_bookmarks().expect("bookmarks"));
    let symbols = |names: &[jayjay_core::RevsetName]| {
        names
            .iter()
            .map(|name| name.symbol.clone())
            .collect::<Vec<_>>()
    };

    assert!(symbols(&vocabulary.bookmarks).contains(&"feature".to_owned()));
    assert!(symbols(&vocabulary.bookmarks).contains(&"\"fix-a|b\"".to_owned()));
    assert!(symbols(&vocabulary.tags).contains(&"v1.0.0".to_owned()));
    assert!(symbols(&vocabulary.aliases).contains(&"wip".to_owned()));
    assert!(symbols(&vocabulary.aliases).contains(&"reviewed(".to_owned()));
}
