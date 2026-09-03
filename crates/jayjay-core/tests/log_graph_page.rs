//! Proves the bounded log/graph page contract: `LogQuery` resolution and the row limit applied after `TopoGroupedGraph`, matching pinned `jj log --limit` semantics.

use jayjay_core::{DEFAULT_LOG_CONTEXT_DEPTH, LogQuery, Repo, build_default_revset};
use jj_test::{build_fork_merge_repo, commit_ids_from_cli_log, init_jj_repo, run_git, run_jj};

fn linear_chain_repo(count: usize) -> (tempfile::TempDir, std::path::PathBuf) {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");
    for i in 0..count {
        run_jj(&["-R", repo_str, "new", "-m", &format!("c{i}")]);
    }
    (temp_dir, repo_path)
}

/// A linear chain with a git tag one commit behind `@` — mirrors a repo whose trunk bookmark sits right
/// next to the working copy: the pinned context depth alone stays sparse, and only widening reaches
/// further back into the tagged commit's own ancestry.
fn tagged_linear_chain_repo(
    before_tag: usize,
    after_tag: usize,
) -> (tempfile::TempDir, std::path::PathBuf) {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");
    for i in 0..before_tag {
        run_jj(&["-R", repo_str, "new", "-m", &format!("c{i}")]);
    }
    run_git(&repo_path, &["tag", "release"]);
    for i in 0..after_tag {
        run_jj(&["-R", repo_str, "new", "-m", &format!("after{i}")]);
    }
    run_jj(&["-R", repo_str, "st"]);
    (temp_dir, repo_path)
}

fn page_ids(repo: &Repo, query: &LogQuery, limit: u32) -> Vec<String> {
    repo.log_graph_page(query, limit)
        .expect("load page")
        .entries
        .into_iter()
        .map(|entry| entry.change.commit_id.id)
        .collect()
}

#[test]
fn default_log_query_falls_back_to_pinned_context_depth_without_override() {
    let (_temp_dir, repo_path) = linear_chain_repo(3);
    let repo = Repo::open(&repo_path).expect("open repo");

    let default_page = repo
        .log_graph_page(&LogQuery::Default, 50)
        .expect("load default page");
    let expected = repo
        .log_graph(&build_default_revset(DEFAULT_LOG_CONTEXT_DEPTH))
        .expect("load pinned expression directly");

    assert_eq!(default_page.entries.len(), expected.len());
}

#[test]
fn default_log_query_widens_context_until_the_page_reaches_the_limit() {
    let (_temp_dir, repo_path) = tagged_linear_chain_repo(28, 1);
    let repo = Repo::open(&repo_path).expect("open repo");

    let narrow = repo
        .log_graph(&build_default_revset(DEFAULT_LOG_CONTEXT_DEPTH))
        .expect("load narrow pinned-depth expression directly");
    assert!(
        narrow.len() < 10,
        "fixture sanity: the tag sits one hop behind @, so context depth {DEFAULT_LOG_CONTEXT_DEPTH} alone \
         should stay sparse, got {}",
        narrow.len()
    );

    let page = repo
        .log_graph_page(&LogQuery::Default, 10)
        .expect("load widened default page");
    assert_eq!(
        page.entries.len(),
        10,
        "the default query should widen past the sparse pinned depth to reach the page size"
    );
}

#[test]
fn default_log_query_stops_widening_once_the_immutable_context_is_exhausted() {
    let (_temp_dir, repo_path) = tagged_linear_chain_repo(28, 1);
    let repo = Repo::open(&repo_path).expect("open repo");

    // 30 commits total (initial change + 28 before the tag + 1 after); a limit above that can never be reached.
    let page = repo
        .log_graph_page(&LogQuery::Default, 100)
        .expect("load default page with an unreachable limit");

    assert!(!page.has_more);
    assert!(page.entries.len() < 100);
}

#[test]
fn revsets_log_override_is_honored_for_default_but_not_explicit() {
    let (_temp_dir, repo_path) = linear_chain_repo(1);
    let repo_str = repo_path.to_str().expect("repo path utf-8");
    run_jj(&[
        "-R",
        repo_str,
        "config",
        "set",
        "--repo",
        "revsets.log",
        "trunk()",
    ]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let default_page = repo
        .log_graph_page(&LogQuery::Default, 50)
        .expect("load default page");
    let trunk_page = repo
        .log_graph_page(&LogQuery::Explicit("trunk()".to_owned()), 50)
        .expect("load explicit trunk page");
    assert_eq!(default_page.entries.len(), trunk_page.entries.len());

    let all_page = repo
        .log_graph_page(&LogQuery::Explicit("all()".to_owned()), 50)
        .expect("load explicit all() page");
    assert!(
        all_page.entries.len() > default_page.entries.len(),
        "an explicit revset must bypass the repository's revsets.log override"
    );
}

#[test]
fn log_graph_page_bounds_entries_to_the_requested_limit() {
    let (_temp_dir, repo_path) = linear_chain_repo(10);
    let repo = Repo::open(&repo_path).expect("open repo");

    let page = repo
        .log_graph_page(&LogQuery::Explicit("all()".to_owned()), 5)
        .expect("load page");

    assert_eq!(page.entries.len(), 5);
    assert_eq!(page.applied_limit, 5);
    assert!(page.has_more);
}

#[test]
fn has_more_is_false_only_when_no_row_follows_the_limit() {
    let (_temp_dir, repo_path) = linear_chain_repo(3);
    let repo = Repo::open(&repo_path).expect("open repo");
    let total = repo
        .log_graph("all()")
        .expect("load full graph for count")
        .len() as u32;

    let exact_page = repo
        .log_graph_page(&LogQuery::Explicit("all()".to_owned()), total)
        .expect("load exact-size page");
    assert_eq!(exact_page.entries.len(), total as usize);
    assert!(!exact_page.has_more);

    let short_page = repo
        .log_graph_page(&LogQuery::Explicit("all()".to_owned()), total - 1)
        .expect("load short page");
    assert_eq!(short_page.entries.len(), (total - 1) as usize);
    assert!(short_page.has_more);
}

#[test]
fn prioritization_is_applied_before_the_row_limit() {
    let (_temp_dir, repo_path) = build_fork_merge_repo();
    let repo_str = repo_path.to_str().expect("repo path utf-8");
    run_jj(&[
        "-R",
        repo_str,
        "config",
        "set",
        "--repo",
        "revsets.log-graph-prioritize",
        "subject(exact:\"C\")",
    ]);
    let repo = Repo::open(&repo_path).expect("open repo");

    let full_cli_order = commit_ids_from_cli_log(repo_str, "all()");
    let limited = page_ids(&repo, &LogQuery::Explicit("all()".to_owned()), 3);

    assert_eq!(limited, full_cli_order[..3]);
}

#[test]
fn increasing_the_limit_preserves_the_earlier_prefix() {
    let (_temp_dir, repo_path) = linear_chain_repo(10);
    let repo = Repo::open(&repo_path).expect("open repo");

    let small = page_ids(&repo, &LogQuery::Explicit("all()".to_owned()), 4);
    let large = page_ids(&repo, &LogQuery::Explicit("all()".to_owned()), 8);

    assert_eq!(small, large[..4]);
}

#[test]
fn log_graph_page_layout_matches_the_returned_entries() {
    let (_temp_dir, repo_path) = linear_chain_repo(3);
    let repo = Repo::open(&repo_path).expect("open repo");

    let page = repo
        .log_graph_page(&LogQuery::Explicit("all()".to_owned()), 2)
        .expect("load page");

    assert_eq!(page.layout.rows.len(), page.entries.len());
}
