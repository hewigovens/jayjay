use std::fs;

use jayjay_core::{MutationEffect, Repo};
use jj_test::{LinearFixture, run_jj_in};

#[test]
fn absorb_distinguishes_no_op_from_applied_changes() {
    let no_op = LinearFixture::build();
    let repo = Repo::open(&no_op.path).expect("open no-op fixture");

    assert_eq!(
        repo.absorb("@").expect("absorb new files"),
        MutationEffect::Unchanged
    );

    let applied = LinearFixture::build();
    run_jj_in(&applied.path, &["bookmark", "delete", "main"]);
    applied.add_tracked_working_copy_edits();
    let repo = Repo::open(&applied.path).expect("open absorbable fixture");

    assert_eq!(
        repo.absorb("@").expect("absorb tracked edits"),
        MutationEffect::Changed
    );
}

#[test]
fn absorb_moves_each_hunk_into_its_ancestor_and_rebases_the_changes_between() {
    let fixture = LinearFixture::build();
    run_jj_in(&fixture.path, &["bookmark", "delete", "main"]);
    fixture.add_tracked_working_copy_edits();
    let repo = Repo::open(&fixture.path).expect("open fixture");

    assert_eq!(repo.absorb("@").expect("absorb"), MutationEffect::Changed);

    let text = |rev: &str, path: &str| repo.file_content(rev, path).expect("read file");
    let readme = "# Sample project\nEdited in GPUI test\n";
    assert_eq!(text("subject(\"initial\")", "README.md"), readme);
    assert_eq!(
        text("subject(\"add feature\")", "feature.txt"),
        "feature\nEdited in GPUI test\n"
    );
    assert_eq!(text("subject(\"add hello\")", "README.md"), readme);
    assert_eq!(text("subject(\"add hello\")", "hello.txt"), "hello\n");
    let changed: Vec<String> = repo
        .show_summary("@")
        .expect("summary")
        .diff
        .into_iter()
        .map(|hunk| hunk.path)
        .collect();
    assert_eq!(changed, ["wip1.txt", "wip2.txt"]);
    assert_eq!(
        fs::read_to_string(fixture.path.join("README.md")).expect("read working copy"),
        readme
    );
}
