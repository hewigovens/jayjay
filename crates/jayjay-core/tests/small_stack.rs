//! Shell worker threads (Swift's cooperative pool, GCD) give Rust 512 KiB of stack; jj-lib's merge and rebase futures must survive that.

use std::fs;

use jayjay_core::Repo;
use jj_test::{init_jj_repo, run_jj_in};

const WORKER_STACK: usize = 512 * 1024;

/// Abandoning a commit with a descendant forces a three-way file merge and a blob write during the rebase: the deepest path a shell can trigger.
#[test]
fn abandon_with_descendant_rebase_survives_a_worker_thread_stack() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let lines = |tag: &str| {
        (0..40)
            .map(|i| format!("line {i} {tag}\n"))
            .collect::<String>()
    };

    fs::write(repo_path.join("a.txt"), lines("base")).expect("write base");
    run_jj_in(&repo_path, &["describe", "-m", "base"]);
    run_jj_in(&repo_path, &["new", "-m", "middle"]);
    fs::write(
        repo_path.join("a.txt"),
        lines("base").replace("line 3 base", "line 3 middle"),
    )
    .expect("write middle");
    run_jj_in(&repo_path, &["new", "-m", "tip"]);
    fs::write(
        repo_path.join("a.txt"),
        lines("base")
            .replace("line 3 base", "line 3 middle")
            .replace("line 30 base", "line 30 tip"),
    )
    .expect("write tip");
    run_jj_in(&repo_path, &["st"]);

    let repo = Repo::open(&repo_path).expect("open repo");
    let middle = repo
        .log("all()")
        .expect("log")
        .into_iter()
        .find(|change| change.description.trim() == "middle")
        .expect("middle change")
        .change_id
        .id;

    std::thread::Builder::new()
        .stack_size(WORKER_STACK)
        .spawn(move || {
            repo.abandon(&middle)
                .expect("abandon on a worker-sized stack")
        })
        .expect("spawn worker")
        .join()
        .expect("worker thread must not crash");

    let content = fs::read_to_string(repo_path.join("a.txt")).expect("read rebased working copy");
    assert!(content.contains("line 3 base"), "{content}");
    assert!(content.contains("line 30 tip"), "{content}");
}
