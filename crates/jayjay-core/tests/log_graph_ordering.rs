//! Proves `Repo::log_graph()` orders commits the same way the pinned `jj` CLI does, since both must feed `jj_lib::graph::TopoGroupedGraph` with the same input.

use jayjay_core::{EdgeType, Repo};
use jj_test::{init_jj_repo, run_jj};

fn commit_ids_from_log_graph(repo: &Repo, revset: &str) -> Vec<String> {
    repo.log_graph(revset)
        .expect("load graph")
        .into_iter()
        .map(|entry| entry.change.commit_id.id)
        .collect()
}

/// Commit IDs in the order the real `jj log` (graph mode) would draw them, extracted from each rendered line rather than via `--no-graph`, since `--no-graph` bypasses `TopoGroupedGraph` entirely and would not be a valid oracle.
fn commit_ids_from_cli_log(repo_str: &str, revset: &str) -> Vec<String> {
    let output = run_jj(&[
        "-R",
        repo_str,
        "log",
        "-r",
        revset,
        "-T",
        "commit_id ++ \"\\n\"",
        "--color",
        "never",
    ]);
    const ROOT_COMMIT_ID: &str = "0000000000000000000000000000000000000000";
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            line.split_whitespace()
                .find(|token| token.len() == 40 && token.bytes().all(|b| b.is_ascii_hexdigit()))
                .map(str::to_owned)
        })
        // jayjay's `should_include_in_log` hides the synthetic root; the CLI does not.
        .filter(|id| id != ROOT_COMMIT_ID)
        .collect()
}

/// Builds a fork-then-merge history: `A` forks into `B` and `C`, then `D` merges them.
/// Returns the repo path only — config must be finalized before `Repo::open`, since `Repo` caches settings from load time rather than re-reading them per call.
fn build_fork_merge_repo() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    run_jj(&["-R", repo_str, "describe", "-m", "A"]);
    run_jj(&["-R", repo_str, "new", "-m", "B"]);
    run_jj(&["-R", repo_str, "new", "subject(exact:\"A\")", "-m", "C"]);
    run_jj(&[
        "-R",
        repo_str,
        "new",
        "subject(exact:\"B\")",
        "subject(exact:\"C\")",
        "-m",
        "D",
    ]);

    (temp_dir, repo_path)
}

#[test]
fn log_graph_matches_cli_order_for_fork_and_merge() {
    let (_temp_dir, repo_path) = build_fork_merge_repo();
    let repo_str = repo_path.to_str().expect("repo path utf-8");
    let repo = Repo::open(&repo_path).expect("open repo");

    let ours = commit_ids_from_log_graph(&repo, "all()");
    let cli = commit_ids_from_cli_log(repo_str, "all()");

    assert_eq!(ours, cli);
}

#[test]
fn log_graph_prioritize_config_emits_configured_branch_first() {
    let (_temp_dir, repo_path) = build_fork_merge_repo();
    let repo_str = repo_path.to_str().expect("repo path utf-8");

    // Without prioritization, B's branch is queued before C's simply because it was authored first; force C's branch to go first via the CLI's own config knob.
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

    let ours = commit_ids_from_log_graph(&repo, "all()");
    let cli = commit_ids_from_cli_log(repo_str, "all()");

    assert_eq!(ours, cli);

    let c_index = ours
        .iter()
        .position(|id| Some(id.as_str()) == cli_commit_for(repo_str, "C").as_deref())
        .expect("C present");
    let b_index = ours
        .iter()
        .position(|id| Some(id.as_str()) == cli_commit_for(repo_str, "B").as_deref())
        .expect("B present");
    assert!(
        c_index < b_index,
        "prioritized branch C should be emitted before B"
    );
}

#[test]
fn log_graph_turns_the_hidden_root_edge_into_a_missing_termination() {
    let (_temp_dir, repo_path) = build_fork_merge_repo();
    let repo = Repo::open(&repo_path).expect("open repo");

    let entries = repo.log_graph("all()").expect("load graph");
    let initial = entries
        .iter()
        .find(|entry| entry.change.description.trim() == "A")
        .expect("initial commit");

    assert_eq!(initial.edges.len(), 1);
    assert_eq!(initial.edges[0].edge_type, EdgeType::Missing);
}

fn cli_commit_for(repo_str: &str, description: &str) -> Option<String> {
    let output = run_jj(&[
        "-R",
        repo_str,
        "log",
        "--no-graph",
        "-r",
        &format!("subject(exact:\"{description}\")"),
        "-T",
        "commit_id",
        "--color",
        "never",
    ]);
    let text = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (!text.is_empty()).then_some(text)
}
