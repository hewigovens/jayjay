use std::fs;
use std::path::{Path, PathBuf};

use jayjay_core::diff::compute_file_diff_full;
use jayjay_core::{ChangeInfo, DiffEditFileSelection, DiffEditRange, Repo};
use tempfile::TempDir;

use crate::{configure_test_user, init_colocated, run_jj_in};

pub fn current_op_id(repo_path: &Path) -> String {
    let output = run_jj_in(repo_path, &["op", "log", "--no-graph", "--limit", "1"]);
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().next())
        .expect("current op id")
        .to_owned()
}

pub fn init_jj_repo() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("create tempdir");
    let repo_path = temp_dir.path().join("repo");

    init_colocated(&repo_path);
    configure_test_user(&repo_path);

    fs::write(repo_path.join("hello.txt"), "hello from jayjay\n").expect("write initial file");
    run_jj_in(&repo_path, &["describe", "-m", "initial change"]);

    temp_dir
}

/// Commit IDs in the order the real `jj log` (graph mode) draws them; `--no-graph` bypasses `TopoGroupedGraph` entirely and would not be a valid ordering oracle.
pub fn commit_ids_from_cli_log(repo_str: &str, revset: &str) -> Vec<String> {
    let output = run_jj_in(
        Path::new(repo_str),
        &[
            "log",
            "-r",
            revset,
            "-T",
            "commit_id ++ \"\\n\"",
            "--color",
            "never",
        ],
    );
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

/// Builds a fork-then-merge history: `A` forks into `B` and `C`, then `D` merges them. Returns the repo path only — config must be finalized before `Repo::open`, since `Repo` caches settings from load time rather than re-reading them per call.
pub fn build_fork_merge_repo() -> (TempDir, PathBuf) {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");

    run_jj_in(&repo_path, &["describe", "-m", "A"]);
    run_jj_in(&repo_path, &["new", "-m", "B"]);
    run_jj_in(&repo_path, &["new", "subject(exact:\"A\")", "-m", "C"]);
    run_jj_in(
        &repo_path,
        &[
            "new",
            "subject(exact:\"B\")",
            "subject(exact:\"C\")",
            "-m",
            "D",
        ],
    );

    (temp_dir, repo_path)
}

pub fn change_by_description<'a>(changes: &'a [ChangeInfo], description: &str) -> &'a ChangeInfo {
    changes
        .iter()
        .find(|change| change.description.trim() == description)
        .unwrap_or_else(|| panic!("missing change with description {description:?}"))
}

fn hunk_for_path(repo: &Repo, rev: &str, path: &str) -> jayjay_core::DiffHunk {
    repo.show(rev)
        .expect("show change")
        .diff
        .into_iter()
        .find(|hunk| hunk.path == path)
        .unwrap_or_else(|| panic!("missing diff for {path} in {rev}"))
}

pub fn whole_file_selection(repo: &Repo, rev: &str, path: &str) -> DiffEditFileSelection {
    let hunk = hunk_for_path(repo, rev, path);
    let old_text = hunk.old.content.as_deref().unwrap_or_default();
    let new_text = hunk.new.content.as_deref().unwrap_or_default();
    let line_count = compute_file_diff_full(path, old_text, new_text, false)
        .lines
        .len() as u32;
    DiffEditFileSelection {
        path: hunk.path,
        old_path: hunk.old_path,
        old_content: hunk.old.content,
        new_content: hunk.new.content,
        hunk_type: hunk.hunk_type,
        line_ranges: vec![DiffEditRange {
            start_line: 1,
            end_line: line_count.max(1),
        }],
    }
}

pub fn selection_for_lines(
    repo: &Repo,
    rev: &str,
    path: &str,
    line_ranges: &[(u32, u32)],
) -> DiffEditFileSelection {
    let hunk = hunk_for_path(repo, rev, path);
    DiffEditFileSelection {
        path: hunk.path,
        old_path: hunk.old_path,
        old_content: hunk.old.content,
        new_content: hunk.new.content,
        hunk_type: hunk.hunk_type,
        line_ranges: line_ranges
            .iter()
            .map(|(start_line, end_line)| DiffEditRange {
                start_line: *start_line,
                end_line: *end_line,
            })
            .collect(),
    }
}

pub fn setup_source_change_with_child() -> (TempDir, PathBuf, Repo) {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");

    fs::write(
        repo_path.join("notes.md"),
        "# moved content\n\nline for diffedit\n",
    )
    .expect("write notes file");
    repo.refresh_working_copy().expect("snapshot source change");
    repo.describe("@", "source change")
        .expect("describe source change");
    repo.new_change("@", "working copy child")
        .expect("create working copy child");

    (temp_dir, repo_path, repo)
}
