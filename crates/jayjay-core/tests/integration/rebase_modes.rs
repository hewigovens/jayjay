use std::collections::HashMap;
use std::path::Path;

use jayjay_core::{RebaseMode, Repo};
use jj_test::{init_jj_repo, run_jj_in};

fn parents_by_description(repo: &Repo) -> HashMap<String, Vec<String>> {
    let changes = repo.log("all()").expect("log");
    let descriptions: HashMap<_, _> = changes
        .iter()
        .map(|c| (c.commit_id.id.clone(), c.description.trim().to_owned()))
        .collect();
    changes
        .iter()
        .filter(|c| !c.description.trim().is_empty())
        .map(|c| {
            let parents = c
                .parents
                .iter()
                .filter_map(|p| descriptions.get(p).cloned())
                .collect();
            (c.description.trim().to_owned(), parents)
        })
        .collect()
}

fn stack_on_trunk(repo_path: &Path, stack: &[&str]) {
    run_jj_in(repo_path, &["describe", "-m", "trunk"]);
    run_jj_in(repo_path, &["bookmark", "create", "main", "-r", "@"]);
    for description in stack {
        run_jj_in(repo_path, &["new", "-m", description]);
    }
}

#[test]
fn branch_mode_moves_the_whole_stack_and_keeps_the_working_copy() {
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    stack_on_trunk(&repo_path, &["base", "work", "tip"]);
    run_jj_in(&repo_path, &["new", "-m", "trunk-next", "main"]);
    run_jj_in(&repo_path, &["bookmark", "set", "main", "-r", "@"]);
    run_jj_in(&repo_path, &["edit", "subject(exact:tip)"]);
    let repo = Repo::open(&repo_path).expect("open repo");
    let op_count = repo.op_log().expect("op log").len();

    repo.rebase("subject(exact:work)", "main", RebaseMode::Branch)
        .expect("rebase the branch onto main");

    let parents = parents_by_description(&repo);
    assert_eq!(parents["base"], ["trunk-next"]);
    assert_eq!(parents["work"], ["base"]);
    assert_eq!(parents["tip"], ["work"]);
    assert_eq!(
        repo.show_summary("@")
            .expect("show @")
            .info
            .description
            .trim(),
        "tip"
    );
    assert_eq!(repo.op_log().expect("op log").len(), op_count + 1);

    repo.rebase("@", "main", RebaseMode::Branch)
        .expect("a branch already on main is a no-op");
    assert_eq!(repo.op_log().expect("op log").len(), op_count + 1);
}
