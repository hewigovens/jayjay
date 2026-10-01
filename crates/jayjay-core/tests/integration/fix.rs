#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};

use jayjay_core::Repo;
use jj_test::{configure_fix_tool, current_op_id, find_change, init_jj_repo, run_jj_in};
use tempfile::TempDir;

const SORT_TOOL: &str = "#!/bin/sh\nsort\n";
const TXT_FILES: &str = r#"["glob:'**/*.txt'"]"#;

/// One change per description on top of the tool's change, each adding an unsorted `<description>.txt`.
fn stack(descriptions: &[&str], patterns: &str, tool: &str) -> (TempDir, PathBuf, Repo) {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    configure_fix_tool(&repo_path, tool, patterns);
    for description in descriptions {
        run_jj_in(&repo_path, &["new", "-m", description]);
        fs::write(repo_path.join(format!("{description}.txt")), "b\na\n")
            .expect("write change file");
        run_jj_in(&repo_path, &["st"]);
    }
    let repo = Repo::open(&repo_path).expect("open repo");
    (temp_dir, repo_path, repo)
}

fn file(repo_path: &Path, name: &str) -> String {
    fs::read_to_string(repo_path.join(name)).expect("read file")
}

#[test]
fn fix_rewrites_a_middle_change_and_keeps_the_fix_in_its_descendants() {
    let (_temp_dir, repo_path, repo) = stack(&["middle", "child", "top"], TXT_FILES, SORT_TOOL);
    let middle = find_change(&repo, "middle");

    let summary = repo
        .fix(std::slice::from_ref(&middle.change_id.id))
        .expect("fix");

    assert_eq!(summary.checked_changes, 3);
    assert_eq!(summary.fixed_changes, 3);
    assert!(summary.failures.is_empty());
    for name in ["middle.txt", "child.txt", "top.txt"] {
        assert_eq!(file(&repo_path, name), "a\nb\n", "{name} was not fixed");
    }
    let rewritten = find_change(&repo, "middle");
    assert_eq!(rewritten.change_id, middle.change_id);
    assert_ne!(rewritten.commit_id, middle.commit_id);
}

#[test]
fn failing_tool_leaves_the_file_and_reports_its_stderr() {
    let (_temp_dir, repo_path, repo) = stack(
        &["target"],
        TXT_FILES,
        "#!/bin/sh\necho 'fixer exploded' >&2\nexit 3\n",
    );
    let target = find_change(&repo, "target");

    let summary = repo
        .fix(std::slice::from_ref(&target.change_id.id))
        .expect("fix");

    assert_eq!(summary.checked_changes, 1);
    assert_eq!(summary.fixed_changes, 0);
    assert_eq!(summary.failures.len(), 1);
    assert_eq!(summary.failures[0].tool, "fixer");
    assert_eq!(summary.failures[0].path, "target.txt");
    assert_eq!(summary.failures[0].message, "fixer exploded");
    assert_eq!(file(&repo_path, "target.txt"), "b\na\n");
}

#[test]
fn pattern_matching_nothing_records_no_operation() {
    let (_temp_dir, repo_path, repo) = stack(&["target"], r#"["glob:'**/*.md'"]"#, SORT_TOOL);
    let target = find_change(&repo, "target");
    let operation = current_op_id(&repo_path);

    let summary = repo
        .fix(std::slice::from_ref(&target.change_id.id))
        .expect("fix");

    assert_eq!(summary.checked_changes, 1);
    assert_eq!(summary.fixed_changes, 0);
    assert!(summary.failures.is_empty());
    assert_eq!(file(&repo_path, "target.txt"), "b\na\n");
    assert_eq!(current_op_id(&repo_path), operation);
}

#[test]
fn line_range_arg_formats_only_changed_lines() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    configure_fix_tool(
        &repo_path,
        "#!/bin/sh\nprintf 'args:%s\\n' \"$*\"\ncat\n",
        TXT_FILES,
    );
    run_jj_in(
        &repo_path,
        &[
            "config",
            "set",
            "--repo",
            "fix.tools.fixer.line-range-arg",
            r#""--lines=$first:$last""#,
        ],
    );
    fs::write(repo_path.join("edited.txt"), "one\ntwo\nthree\n").expect("write base file");
    fs::write(repo_path.join("trimmed.txt"), "one\ntwo\nthree\n").expect("write base file");
    run_jj_in(&repo_path, &["new"]);
    fs::write(repo_path.join("edited.txt"), "one\nTWO\nthree\n").expect("edit file");
    fs::write(repo_path.join("trimmed.txt"), "one\nthree\n").expect("trim file");
    let repo = Repo::open(&repo_path).expect("open repo");

    repo.fix(&["@".to_owned()]).expect("fix");

    assert_eq!(
        file(&repo_path, "edited.txt"),
        "args:--lines=2:2\none\nTWO\nthree\n"
    );
    assert_eq!(
        file(&repo_path, "trimmed.txt"),
        "one\nthree\n",
        "a change that only deletes lines has nothing to format"
    );
}

#[test]
fn immutable_root_is_refused() {
    let (_temp_dir, repo_path, _repo) = stack(&["target"], TXT_FILES, SORT_TOOL);
    run_jj_in(
        &repo_path,
        &[
            "config",
            "set",
            "--repo",
            r#"revset-aliases."immutable_heads()""#,
            r#"trunk() | subject(exact:"target")"#,
        ],
    );
    let repo = Repo::open(&repo_path).expect("reopen repo");
    let target = find_change(&repo, "target");

    let error = repo
        .fix(std::slice::from_ref(&target.change_id.id))
        .expect_err("an immutable root must be refused");

    assert!(
        error
            .to_string()
            .ends_with("is immutable and cannot be rewritten"),
        "{error}"
    );
    assert_eq!(file(&repo_path, "target.txt"), "b\na\n");
}
