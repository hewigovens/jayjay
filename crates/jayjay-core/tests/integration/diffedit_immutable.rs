use std::fs;

use jayjay_core::{DiffEditDestination, Repo};
use jj_test::{init_jj_repo, run_git, run_jj_in, whole_file_selection};

#[test]
fn diffedit_refuses_every_destination_for_an_immutable_source() {
    for destination in [
        DiffEditDestination::RemoveFromSource,
        DiffEditDestination::MoveToWorkingCopy,
        DiffEditDestination::NewChild,
        DiffEditDestination::NewParallel,
    ] {
        let temp_dir = init_jj_repo();
        let repo_path = temp_dir.path().join("repo");
        fs::write(repo_path.join("notes.md"), "selected\nrest\n").expect("write source");
        run_jj_in(&repo_path, &["describe", "-m", "protected"]);
        run_jj_in(&repo_path, &["new", "-m", "child"]);
        let repo = Repo::open(&repo_path).expect("open repo");
        let selection = whole_file_selection(&repo, "@-", "notes.md");
        run_git(&repo_path, &["tag", "release"]);
        run_jj_in(&repo_path, &["st"]);

        let err = repo
            .apply_diff_selection("@-", destination, &[selection], "selected", false)
            .expect_err("immutable source must not be rewritten");
        assert!(
            err.to_string().contains("immutable"),
            "unclear error for {destination:?}: {err}"
        );
        assert_eq!(
            repo.file_content("@-", "notes.md")
                .expect("immutable file remains")
                .trim_end(),
            "selected\nrest"
        );
    }
}
